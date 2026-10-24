#!/usr/bin/env python3
"""Build and execute a native C11 consumer of both C library formats (REQ-H-03).

Only existing compilers are used. Windows requires an MSVC Rust host and Visual Studio
C tools/SDK, either a Developer Command Prompt (cl) or existing clang-cl/clang with MSVC
support. Missing SDK libraries are an environment failure, not a successful verification.
No compiler, SDK or target is installed. Native execution proves only the current platform.
Sources: https://doc.rust-lang.org/reference/linkage.html and
https://doc.rust-lang.org/rustc/command-line-arguments.html#--print-print-compiler-information
"""

import json
import os
from pathlib import Path
import re
import shlex
import shutil
import subprocess
import sys
import tempfile

ROOT = Path(__file__).resolve().parents[1]
CRATE = ROOT / "crates" / "sisyphium-limen"


def run(arguments, *, env=None, cwd=ROOT):
    """Execute structured arguments and retain diagnostics for failed verification."""
    result = subprocess.run([str(arg) for arg in arguments], cwd=cwd, env=env,
                            text=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                            check=False)
    if result.returncode:
        raise RuntimeError(f"command failed ({result.returncode}): {arguments!r}\n"
                           f"{result.stdout}{result.stderr}")
    return result


def compiler(host):
    """Choose an existing native C driver without mutating the machine (REQ-H-03)."""
    if os.name == "nt":
        if "windows-msvc" not in host:
            raise RuntimeError("Windows verification requires a native MSVC Rust toolchain.")
        if shutil.which("cl"):
            return [shutil.which("cl")]
        if shutil.which("clang-cl"):
            return [shutil.which("clang-cl"), f"--target={host}"]
        if shutil.which("clang"):
            return [shutil.which("clang"), "--driver-mode=cl", f"--target={host}"]
        raise RuntimeError("No C compiler found. Run in an existing Visual Studio Developer "
                           "Command Prompt with cl and the Windows SDK, or configure existing clang-cl.")
    configured = shlex.split(os.environ.get("CC", "cc"))
    if not configured or not shutil.which(configured[0]):
        raise RuntimeError("No existing C compiler found; configure CC for a C11 compiler.")
    return configured


def main():
    """Compare independently produced core fixtures through both native linking modes."""
    version = run(["rustc", "-vV"]).stdout
    match = re.search(r"^host: (.+)$", version, re.MULTILINE)
    if not match:
        raise RuntimeError("rustc did not report its native host triple")
    host = match.group(1)
    cc = compiler(host)
    metadata = json.loads(run(["cargo", "metadata", "--no-deps", "--format-version", "1"]).stdout)
    package = next(p for p in metadata["packages"] if p["name"] == "sisyphium-limen")
    components = [int(value) for value in package["version"].split("-")[0].split("+")[0].split(".")]
    if len(components) != 3 or any(not 0 <= value <= 255 for value in components):
        raise RuntimeError("ABI package version requires three eight-bit components")
    packed = components[0] << 16 | components[1] << 8 | components[2]
    with tempfile.TemporaryDirectory(prefix="sisyphium-ffi-") as directory:
        temporary = Path(directory)
        target = temporary / "target"
        common = ["-p", "sisyphium-limen", "--target", host, "--target-dir", target]
        # A fresh target forces rustc to emit the native link list even on repeated runs.
        build = run(["cargo", "rustc", *common, "--lib", "--", "--print=native-static-libs"])
        native = re.search(r"native-static-libs:\s*([^\r\n]+)", build.stderr + build.stdout)
        if not native:
            raise RuntimeError("rustc did not report native-static-libs; cannot validate static linkage")
        native_libraries = shlex.split(native.group(1), posix=os.name != "nt")
        fixture = run(["cargo", "run", *common, "--quiet", "--example", "fixture"]).stdout
        (temporary / "fixture.h").write_text(fixture, encoding="utf-8")
        artifacts = target / host / "debug"
        environment = os.environ.copy()
        if os.name == "nt":
            dynamic = artifacts / "sisyphium_limen.dll.lib"
            static = artifacts / "sisyphium_limen.lib"
            dll = artifacts / "sisyphium_limen.dll"
            if not dll.is_file():
                raise RuntimeError(f"Missing cdylib: {dll}")
            # Executable-local lookup avoids accidental resolution to a different installation.
            shutil.copy2(dll, temporary / dll.name)
        else:
            suffix = ".dylib" if sys.platform == "darwin" else ".so"
            dynamic = artifacts / f"libsisyphium_limen{suffix}"
            static = artifacts / "libsisyphium_limen.a"
            variable = "DYLD_LIBRARY_PATH" if sys.platform == "darwin" else "LD_LIBRARY_PATH"
            environment[variable] = str(artifacts)
        for mode, library in [("cdylib", dynamic), ("staticlib", static)]:
            if not library.is_file():
                raise RuntimeError(f"Missing {mode} artifact: {library}")
            executable = temporary / (mode + (".exe" if os.name == "nt" else ""))
            native_args = native_libraries if mode == "staticlib" else []
            if os.name == "nt":
                arguments = [*cc, "/nologo", "/std:c11", "/W4", "/WX", "/MD",
                             f"/I{CRATE / 'include'}", f"/I{temporary}",
                             f"/DEXPECTED_VERSION={packed}", str(CRATE / "tests" / "consumer.c"),
                             f"/Fe:{executable}", f"/Fo:{temporary / (mode + '.obj')}",
                             "/link", str(library), *native_args]
            else:
                arguments = [*cc, "-std=c11", "-Wall", "-Wextra", "-Werror", "-pedantic",
                             "-I", CRATE / "include", "-I", temporary,
                             f"-DEXPECTED_VERSION={packed}", CRATE / "tests" / "consumer.c",
                             library, *native_args, "-o", executable]
            try:
                run(arguments, cwd=temporary)
            except RuntimeError as error:
                if os.name == "nt":
                    raise RuntimeError(f"{error}\nMSVC requires the matching C tools and Windows SDK; "
                                       "use the existing Developer Command Prompt or configure clang's "
                                       "MSVC driver and SDK search paths. No tools were installed.") from error
                raise
            first = run([executable], env=environment, cwd=temporary).stdout
            second = run([executable], env=environment, cwd=temporary).stdout
            if first != second:
                raise RuntimeError(f"Nondeterministic C consumer output for {mode}")
            print(f"{host} {mode}: {first.strip()}")
        print("ffi: native cdylib and staticlib executions passed")


if __name__ == "__main__":
    try:
        main()
    except (OSError, ValueError, RuntimeError, StopIteration) as error:
        print(f"ffi: {error}", file=sys.stderr)
        sys.exit(1)
