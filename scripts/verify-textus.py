#!/usr/bin/env python3
"""Verify native terminal shutdown in exclusively owned children (REQ-D-02, REQ-H-02).

Unix uses newly allocated PTYs and signals only the PIDs returned by this script.
Windows starts a controller with CREATE_NEW_CONSOLE, checks its console membership,
then sends BREAK to a fresh child process group and C only inside that private console.
No AttachConsole, user console event, wallpaper command, installation or global mode
change is performed. Missing native prerequisites fail the verification; none are skipped.

Sources:
https://learn.microsoft.com/en-us/windows/console/generateconsolectrlevent
https://learn.microsoft.com/en-us/windows/console/creation-of-a-console
https://learn.microsoft.com/en-us/windows/console/setconsolectrlhandler
https://learn.microsoft.com/en-us/windows/win32/procthread/process-creation-flags
https://learn.microsoft.com/en-us/windows/console/getconsoleprocesslist
https://learn.microsoft.com/en-us/windows/console/getconsolemode
"""

from contextlib import contextmanager
import json
import os
from pathlib import Path
import re
import signal
import subprocess
import sys
import tempfile
import time

ROOT = Path(__file__).resolve().parents[1]
TIMEOUT = 15
CAPTURE_LIMIT = 131_072
PRIVATE_MARKER = "SISYPHIUM_PRIVATE_CONSOLE_PARENT"


def require(condition, message):
    """Keep every proof active even when Python runs with optimization (REQ-D-02)."""
    if not condition:
        raise RuntimeError(message)


def read_capture(path):
    """Bound memory used by captured native output, including broken implementations."""
    with path.open("rb") as stream:
        content = stream.read(CAPTURE_LIMIT + 1)
    require(len(content) <= CAPTURE_LIMIT, f"capture exceeds {CAPTURE_LIMIT} bytes: {path.name}")
    return content


@contextmanager
def child(arguments, directory, *, env=None, creationflags=0):
    """Own a child and its captures; forced termination is cleanup, never signal evidence."""
    stdout = directory / "stdout"
    stderr = directory / "stderr"
    with stdout.open("wb") as output, stderr.open("wb") as errors:
        process = subprocess.Popen([str(value) for value in arguments], cwd=directory,
                                   stdin=subprocess.DEVNULL, stdout=output, stderr=errors,
                                   env=env, creationflags=creationflags,
                                   start_new_session=os.name != "nt")
        try:
            yield process, stdout, stderr
        finally:
            if process.poll() is None:
                process.kill()
                process.wait(timeout=5)


def wait_for(process, path, predicate, label):
    """Wait on flushed evidence instead of guessing native registration timing (REQ-D-02)."""
    deadline = time.monotonic() + TIMEOUT
    while time.monotonic() < deadline:
        content = read_capture(path)
        if predicate(content):
            return content
        require(process.poll() is None, f"child exited before {label}: {content!r}")
        time.sleep(0.025)
    raise RuntimeError(f"timed out waiting for {label}")


def finish(process, stdout, stderr):
    """Require normal native exit and retain bounded diagnostics on failure (REQ-D-02)."""
    code = process.wait(timeout=TIMEOUT)
    output, errors = read_capture(stdout), read_capture(stderr)
    require(code == 0, f"native child failed ({code}): {errors.decode('utf-8', 'replace')}")
    require(not errors, f"unexpected native stderr: {errors!r}")
    return output


def state_is_complete(directory):
    """Require exactly32 hexadecimal bytes and absence of lock/temporary artifacts (REQ-D-02)."""
    with (directory / "world").open("rb") as stream:
        data = stream.read(33)
    require(re.fullmatch(rb"[0-9a-fA-F]{32}", data) is not None,
            "shutdown did not persist exactly32 hexadecimal bytes")
    require({path.name for path in directory.iterdir()} == {"world", "stdout", "stderr"},
            "shutdown left an unexpected lock or temporary artifact")


def frame_arguments(binary, state=None):
    """Wait in a sixty-second regimen so a successful short exit requires real shutdown."""
    args = [binary, "--plain", "--frames", "2", "--cols", "4", "--rows", "2",
            "--max-hz", "0", "--energy", "0", "--ticks", "0", "--seed", "17"]
    if state is not None:
        args.extend(["--state", state])
    return args


def verify_finite(binary):
    """Verify finite plain geometry and pipe fallback without terminal interaction (REQ-H-02)."""
    for frames in (None, 2):
        with tempfile.TemporaryDirectory(prefix="sisyphium-textus-finite-") as temporary:
            directory = Path(temporary)
            args = [binary, "--cols", "40", "--rows", "12", "--ticks", "0"]
            if frames is not None:
                args.extend(["--plain", "--frames", str(frames)])
            with child(args, directory) as (process, stdout, stderr):
                output = finish(process, stdout, stderr)
            require(b"\x1b" not in output, "plain/redirected output emitted ANSI")
            lines = output.decode("utf-8").splitlines()
            expected = frames or 1
            require(len(lines) == expected * 12 + expected - 1, "finite frame row count differs")
            require(sum(len(line) == 40 for line in lines) == expected * 12,
                    "finite frame column count differs")
            require({path.name for path in directory.iterdir()} == {"stdout", "stderr"},
                    "state omitted but native host created files")
    print("textus: finite plain frames and redirected default passed", flush=True)


def verify_guard(probe, send, *, creationflags=0, console_check=None):
    """Prove removal/restoration by delivering two events while the same child lives (REQ-D-02)."""
    with tempfile.TemporaryDirectory(prefix="sisyphium-textus-guard-") as temporary:
        directory = Path(temporary)
        with child([probe], directory, creationflags=creationflags) as (process, stdout, stderr):
            wait_for(process, stdout, lambda data: b"READY\n" in data, "guard readiness")
            if console_check:
                console_check(process)
            send(process)
            wait_for(process, stdout, lambda data: b"RESTORED\n" in data, "guard restoration")
            require(process.poll() is None, "guard proof ended before the second event")
            if console_check:
                console_check(process)
            send(process)
            output = finish(process, stdout, stderr)
            marker = (b"PREVIOUS_HANDLER_OBSERVED MODES_PRESERVED\n" if os.name == "nt"
                      else b"PREVIOUS_HANDLER_OBSERVED\n")
            require(marker in output, "previous handler was not observed after restoration")


def verify_unix_signal(binary, probe, event):
    """Signal only an owned child PID during the long native sleep (REQ-D-02)."""
    with tempfile.TemporaryDirectory(prefix="sisyphium-textus-signal-") as temporary:
        directory = Path(temporary)
        with child(frame_arguments(binary, directory / "world"), directory) as (process, stdout, stderr):
            wait_for(process, stdout, lambda data: data.count(b"\n") >= 2, "first frame")
            start = time.monotonic()
            process.send_signal(event)
            output = finish(process, stdout, stderr)
            elapsed = time.monotonic() - start
            require(output.count(b"\n") == 2, "shutdown presented an extra frame")
            require(b"\x1b" not in output, "plain shutdown output emitted ANSI")
            state_is_complete(directory)
    verify_guard(probe, lambda process: process.send_signal(event))
    print(f"textus: {event.name} saved32, released lock, restored live prior handler "
          f"({elapsed:.3f}s observed)", flush=True)


def verify_pty(binary, plain):
    """Compare termios on a newly allocated PTY; no existing terminal is opened (REQ-H-02)."""
    import errno
    import pty
    import select
    import termios

    master, slave = pty.openpty()
    before = termios.tcgetattr(slave)
    process = None
    try:
        with tempfile.TemporaryDirectory(prefix="sisyphium-textus-pty-") as temporary:
            directory = Path(temporary)
            with (directory / "stderr").open("wb") as errors:
                args = [binary, "--frames", "2", "--cols", "4", "--rows", "2", "--ticks", "0"]
                if plain:
                    args.append("--plain")
                process = subprocess.Popen([str(value) for value in args], cwd=directory,
                                           stdin=slave, stdout=slave, stderr=errors,
                                           env={**os.environ, "TERM": "xterm-256color"},
                                           start_new_session=True)
                output = bytearray()
                deadline = time.monotonic() + TIMEOUT
                while time.monotonic() < deadline:
                    ready, _, _ = select.select([master], [], [], 0.025)
                    if ready:
                        try:
                            data = os.read(master, 4096)
                        except OSError as error:
                            if error.errno != errno.EIO:
                                raise
                            break
                        if not data:
                            break
                        output.extend(data)
                        require(len(output) <= CAPTURE_LIMIT, "PTY output exceeded its limit")
                    elif process.poll() is not None:
                        break
                require(process.wait(timeout=1) == 0, "PTY child did not exit normally")
            require(not read_capture(directory / "stderr"), "PTY child emitted diagnostics")
            require(termios.tcgetattr(slave) == before, "PTY attributes changed")
            if plain:
                require(b"\x1b" not in output, "plain PTY output emitted ANSI")
            else:
                require(b"\x1b[H" in output and b"\x1b[0m" in output,
                        "ANSI PTY did not emit between-frame home and color reset")
                allowed = rb"\x1b\[(?:(?:38|48);2;[0-9]{1,3};[0-9]{1,3};[0-9]{1,3}m|0m|H)"
                remainder = re.sub(allowed, b"", output)
                require(b"\x1b" not in remainder, "terminal emitted a non-permitted ANSI sequence")
                require(len(remainder.decode("utf-8").splitlines()) == 4,
                        "ANSI PTY did not preserve two frames of two rows")
            require({path.name for path in directory.iterdir()} == {"stderr"},
                    "PTY run created a state file without --state")
    finally:
        if process is not None and process.poll() is None:
            process.kill()
            process.wait(timeout=5)
        os.close(master)
        os.close(slave)
    print(f"textus: private PTY {'plain' if plain else 'ANSI'} preserved termios", flush=True)


def windows_api():
    """Declare authoritative DWORD/BOOL/HANDLE signatures only on native Windows."""
    import ctypes
    from ctypes import wintypes

    kernel = ctypes.WinDLL("kernel32", use_last_error=True)
    callback = ctypes.WINFUNCTYPE(wintypes.BOOL, wintypes.DWORD)
    kernel.GetConsoleProcessList.argtypes = [ctypes.POINTER(wintypes.DWORD), wintypes.DWORD]
    kernel.GetConsoleProcessList.restype = wintypes.DWORD
    kernel.GenerateConsoleCtrlEvent.argtypes = [wintypes.DWORD, wintypes.DWORD]
    kernel.GenerateConsoleCtrlEvent.restype = wintypes.BOOL
    kernel.SetConsoleCtrlHandler.argtypes = [callback, wintypes.BOOL]
    kernel.SetConsoleCtrlHandler.restype = wintypes.BOOL
    kernel.GetConsoleMode.argtypes = [wintypes.HANDLE, ctypes.POINTER(wintypes.DWORD)]
    kernel.GetConsoleMode.restype = wintypes.BOOL
    return ctypes, wintypes, kernel, callback


def console_members(api):
    """Refuse any console containing a process outside the exact owned set (REQ-D-02)."""
    ctypes, wintypes, kernel, _ = api
    members = (wintypes.DWORD * 64)()
    count = kernel.GetConsoleProcessList(members, len(members))
    if not count:
        raise ctypes.WinError(ctypes.get_last_error())
    require(count <= len(members), "unexpectedly large console; control event refused")
    return set(members[:count])


def console_modes(api):
    """Read both console modes without changing either of them (REQ-H-02)."""
    import msvcrt
    ctypes, wintypes, kernel, _ = api
    result = []
    for name in ("CONIN$", "CONOUT$"):
        descriptor = os.open(name, os.O_RDWR)
        try:
            mode = wintypes.DWORD()
            if not kernel.GetConsoleMode(msvcrt.get_osfhandle(descriptor), ctypes.byref(mode)):
                raise ctypes.WinError(ctypes.get_last_error())
            result.append(mode.value)
        finally:
            os.close(descriptor)
    return tuple(result)


def private_console_controller(binary, probe):
    """Generate real C/BREAK only in the new console created for this controller (REQ-D-02)."""
    require(os.name == "nt", "private console controller requires Windows")
    parent = os.environ.get(PRIVATE_MARKER)
    require(parent == str(os.getppid()) and parent != str(os.getpid()),
            "missing or mismatched private-console parent marker")
    api = windows_api()
    ctypes, _, kernel, callback_type = api
    require(console_members(api) == {os.getpid()}, "controller did not receive an exclusive new console")

    # A callable handler protects this controller without setting the inheritable
    # ignore-Ctrl-C attribute. Children must receive C so their real handlers are tested.
    callback = callback_type(lambda event: 1 if event in (0, 1) else 0)
    null_callback = callback_type()
    if not kernel.SetConsoleCtrlHandler(null_callback, False):
        raise ctypes.WinError(ctypes.get_last_error())
    if not kernel.SetConsoleCtrlHandler(callback, True):
        raise ctypes.WinError(ctypes.get_last_error())

    def check(process):
        """Validate the exact console population immediately before each event."""
        require(process.poll() is None, "target child exited before its event")
        require(console_members(api) == {os.getpid(), process.pid},
                "console contains an unowned process; control event refused")

    try:
        for event, name in ((1, "CTRL_BREAK"), (0, "CTRL_C")):
            flags = subprocess.CREATE_NEW_PROCESS_GROUP if event == 1 else 0

            def send(process):
                """Use targeted BREAK or private-console C broadcast as documented."""
                check(process)
                group = process.pid if event == 1 else 0
                if not kernel.GenerateConsoleCtrlEvent(event, group):
                    raise ctypes.WinError(ctypes.get_last_error())

            before = console_modes(api)
            with tempfile.TemporaryDirectory(prefix="sisyphium-textus-win-signal-") as temporary:
                directory = Path(temporary)
                with child(frame_arguments(binary, directory / "world"), directory,
                           creationflags=flags) as (process, stdout, stderr):
                    wait_for(process, stdout, lambda data: data.count(b"\n") >= 2, "first Windows frame")
                    send(process)
                    output = finish(process, stdout, stderr)
                    require(output.count(b"\n") == 2 and b"\x1b" not in output,
                            "Windows shutdown changed plain geometry or emitted another frame")
                    state_is_complete(directory)
            require(console_modes(api) == before, "native terminal run changed console modes")
            require(console_members(api) == {os.getpid()}, "native child remains attached after exit")
            verify_guard(probe, send, creationflags=flags, console_check=check)
            require(console_modes(api) == before, "guard probe changed console modes")
            require(console_members(api) == {os.getpid()}, "probe remains attached after exit")
            print(f"textus: {name} saved32, released lock, restored live prior handler, "
                  "preserved private console modes", flush=True)
    finally:
        if not kernel.SetConsoleCtrlHandler(callback, False):
            raise ctypes.WinError(ctypes.get_last_error())


def verify_windows(binary, probe):
    """Create the isolated console before permitting any control-event generation."""
    with tempfile.TemporaryDirectory(prefix="sisyphium-textus-win-controller-") as temporary:
        directory = Path(temporary)
        environment = {**os.environ, PRIVATE_MARKER: str(os.getpid())}
        args = [sys.executable, Path(__file__).resolve(), "--private-console-controller", binary, probe]
        with child(args, directory, env=environment,
                   creationflags=subprocess.CREATE_NEW_CONSOLE) as (process, stdout, stderr):
            code = process.wait(timeout=45)
            output, errors = read_capture(stdout), read_capture(stderr)
            require(code == 0, f"private Windows console proof failed ({code}): "
                    f"{errors.decode('utf-8', 'replace')}")
            require(not errors, f"private controller emitted diagnostics: {errors!r}")
            require(b"CTRL_BREAK saved32" in output and b"CTRL_C saved32" in output,
                    "private controller did not prove both native control events")
            print(output.decode("utf-8"), end="", flush=True)


def build_native():
    """Build only the native CLI and probe using the existing toolchain (REQ-D-02)."""
    version = subprocess.run(["rustc", "-vV"], cwd=ROOT, capture_output=True,
                             text=True, check=True).stdout
    match = re.search(r"^host: (.+)$", version, re.MULTILINE)
    require(match is not None, "rustc did not report its native host")
    host = match.group(1)
    if os.name == "nt":
        require("windows-msvc" in host, "Windows proof requires a native MSVC Rust host")
    else:
        require(sys.platform in ("darwin", "linux") and
                host.startswith(("x86_64-", "aarch64-")) and
                ("apple-darwin" in host or "linux-gnu" in host),
                f"native signal ABI not admitted: {host}")
    metadata = subprocess.run(["cargo", "metadata", "--no-deps", "--format-version", "1"],
                              cwd=ROOT, capture_output=True, text=True, check=True)
    target = Path(json.loads(metadata.stdout)["target_directory"]) / host / "debug"
    subprocess.run(["cargo", "build", "-p", "sisyphium-textus", "--target", host,
                    "--bin", "sisyphium-textus", "--example", "custodia_probe"],
                   cwd=ROOT, check=True)
    suffix = ".exe" if os.name == "nt" else ""
    binary = target / ("sisyphium-textus" + suffix)
    probe = target / "examples" / ("custodia_probe" + suffix)
    require(binary.is_file() and probe.is_file(), "native CLI/probe artifact is missing")
    return binary, probe, host


def main():
    """Require the current platform's full native evidence; no success-by-skip path exists."""
    if len(sys.argv) == 4 and sys.argv[1] == "--private-console-controller":
        private_console_controller(Path(sys.argv[2]), Path(sys.argv[3]))
        return
    require(len(sys.argv) == 1, "verify-textus.py takes no public arguments")
    binary, probe, host = build_native()
    verify_finite(binary)
    if os.name == "nt":
        verify_windows(binary, probe)
    else:
        for event in (signal.SIGINT, signal.SIGTERM):
            verify_unix_signal(binary, probe, event)
        for plain in (False, True):
            verify_pty(binary, plain)
    print(f"textus: all required native proofs passed on {host}", flush=True)


if __name__ == "__main__":
    try:
        main()
    except (OSError, ValueError, RuntimeError, subprocess.SubprocessError) as error:
        print(f"textus: {error}", file=sys.stderr)
        sys.exit(1)
