#!/usr/bin/env python3
"""Verify adapter boundaries with isolated homes and fake native commands only."""
import argparse
import contextlib
import errno
import hashlib
import re
import json
import os
from pathlib import Path
import plistlib
import platform
import shlex
import shutil
import subprocess
import sys
import tempfile
import struct
import zlib
import xml.etree.ElementTree as ET

ROOT = Path(__file__).resolve().parents[1]
# Reviewed source snapshots make a new executable path an explicit review event.
# Update these hashes only after independently reviewing every host command boundary.
REVIEWED_HOSTS = {'communis.sh': '89d61364abc096f4aa78652c6a1547a65b2e0734c86635814303352354b184c2', 'macos/sisyphium.plist.in': 'e7f2d01975d1f8d442a5f6b4620d8a5d65baab621c1369c2f41fd5cc9412e2b6', 'macos/sisyphium.sh': '754e9bbcb726ed39659f3bbba7d54c3bc15176827a7134a525914b80f8d0198f', 'linux/sisyphium.timer.in': '06999d8ab8f099b4f645989338d18bb4ad533af794a7efce089d94f060cd0cca', 'linux/sisyphium.sh': 'd4dfe5a09696594a5dfabd9faca65dd155510b1b262c75ff67b17477317b80e8', 'linux/sisyphium.service.in': '7a2b4fd84db1c4f643334bd17b28afc3231913cda2ec02a0cf29dffdaad6285b', 'windows/sisyphium-task.xml.in': '53138c6d7795d2122882a8bb08def663354453bca8686a6fe8c9862352bea0bf', 'windows/sisyphium.ps1': 'd937861b843693fa03489c3ad8878e68890ec1c6551d8b51f84587e1df256111'}
INERT_UTILITIES = ('sh','dirname','sed','grep','awk','wc','tr','od','mkdir','rm','cat','id','iconv')
FAKE_COMMANDS = ('tabula','pmset','system_profiler','sw_vers','osascript','launchctl',
                 'plutil','systemctl','feh','swww','xrandr','swaymsg')
DANGEROUS_COMMANDS = ('osascript','launchctl','systemctl','feh','swww')


def require_guard(condition, message):
    """Safety checks remain enabled even when Python runs with optimization."""
    if not condition:
        raise RuntimeError('host isolation guard: ' + message)


@contextlib.contextmanager
def retained_workspace(system):
    """Retain every artifact, including on failure; never delete incident evidence."""
    location = os.environ.get('SISYPHIUM_HOST_TEST_ROOT') or os.environ.get('CARGO_TARGET_TMPDIR')
    parent = Path(location).expanduser().absolute() if location else ROOT / 'out' / 'host-tests'
    parent.mkdir(parents=True, exist_ok=True)
    base = Path(tempfile.mkdtemp(prefix=system + '-', dir=parent)).resolve()
    base.chmod(0o700)
    (base / '.owner').write_text(base.name, encoding='utf-8')
    print('host test artifacts: ' + str(base), flush=True)
    try:
        yield base
    except BaseException as error:
        (base / 'result.json').write_text(json.dumps({'result':'failed','error':repr(error)},ensure_ascii=True), encoding='utf-8')
        raise
    else:
        (base / 'result.json').write_text('{"result":"passed"}', encoding='utf-8')
    finally:
        # Logs can contain local paths. Restrict regular files without following links.
        for file in base.rglob('*'):
            if file.is_file() and not file.is_symlink():
                file.chmod(0o700 if file.stat().st_mode & 0o111 else 0o600)


def isolated_environment(base, home, commands):
    """Do not inherit shell startup hooks, user modules, or fake failure controls."""
    keys = ('SystemRoot','WINDIR','COMSPEC','PATHEXT','PROCESSOR_ARCHITECTURE','NUMBER_OF_PROCESSORS')
    env = {key:os.environ[key] for key in keys if key in os.environ}
    temporary = base / 'tmp'
    temporary.mkdir()
    env.update(HOME=str(home), USERPROFILE=str(home), LOCALAPPDATA=str(home),
               APPDATA=str(home / 'appdata'), XDG_CONFIG_HOME=str(home / 'config'),
               XDG_STATE_HOME=str(home / 'state-root'), XDG_CACHE_HOME=str(home / 'cache'),
               XDG_DATA_HOME=str(home / 'data'), XDG_RUNTIME_DIR=str(temporary),
               TMPDIR=str(temporary), TMP=str(temporary), TEMP=str(temporary),
               PATH=str(commands), HOST_ROOT=str(base), HOST_OWNER=base.name,
               HOST_LOG=str(base / 'commands.jsonl'), HOST_BEFORE=str(base / 'before.json'), HOST_PNG=PNG.hex(),
               PYTHONDONTWRITEBYTECODE='1', LC_ALL='C',
               # PowerShell reads this before startup and may write the cache in
               # the background after module import. Keep it outside the HOME
               # whose dry-run immutability is asserted below.
               PSModuleAnalysisCachePath=str(temporary / 'ModuleAnalysisCache'))
    return env


def guard_environment(base, env, commands):
    root = base.resolve()
    require_guard(env['PATH'] == str(commands), 'PATH contains unreviewed directories')
    for key in ('HOME','USERPROFILE','LOCALAPPDATA','APPDATA','XDG_CONFIG_HOME','XDG_STATE_HOME',
                'XDG_CACHE_HOME','XDG_DATA_HOME','XDG_RUNTIME_DIR','TMPDIR','TMP','TEMP','HOST_LOG',
                'PSModuleAnalysisCachePath','HOST_BEFORE'):
        path = Path(env[key]).resolve()
        require_guard(root in path.parents, key + ' escapes test root')
    require_guard(Path(env['HOST_BEFORE']).resolve() == root / 'before.json',
                  'pre-product snapshot escaped dedicated path')
    cache = Path(env['PSModuleAnalysisCachePath']).resolve()
    require_guard(cache == root / 'tmp' / 'ModuleAnalysisCache', 'module cache escaped dedicated temporary path')
    require_guard(Path(env['HOME']).resolve() not in cache.parents, 'module cache overlaps observed HOME')
    require_guard(env['HOST_OWNER'] == (base / '.owner').read_text(), 'wrong artifact owner')


def reviewed_copy(destination):
    for relative, expected in REVIEWED_HOSTS.items():
        source = ROOT / 'hosts' / relative
        require_guard(hashlib.sha256(source.read_bytes()).hexdigest() == expected,
                      'host changed since command review: ' + relative)
    shutil.copytree(ROOT / 'hosts', destination)
    snapshot = {}
    for relative in REVIEWED_HOSTS:
        path = destination / relative
        content = path.read_bytes()
        require_guard(hashlib.sha256(content).hexdigest() == REVIEWED_HOSTS[relative],
                      'copied host differs from reviewed source: ' + relative)
        if path.suffix == '.sh':
            text = content.decode('utf-8')
            for command in DANGEROUS_COMMANDS:
                require_guard(not re.search(r'[/\\][^\s\n]*' + re.escape(command) + r'\b', text),
                              'absolute or qualified desktop command: ' + relative)
        snapshot[path] = hashlib.sha256(content).hexdigest()
    return snapshot


def guard_snapshot(snapshot):
    for path, expected in snapshot.items():
        require_guard(not path.is_symlink() and hashlib.sha256(path.read_bytes()).hexdigest() == expected,
                      'copied host changed after preflight: ' + str(path))


def preflight_unix(base, commands, env, absent=()):
    guard_environment(base, env, commands)
    require_guard(set(path.name for path in commands.iterdir()) == set(INERT_UTILITIES) | (set(FAKE_COMMANDS) - set(absent)),
                  'unexpected command in closed PATH')
    for name in INERT_UTILITIES:
        path = commands / name
        allowed = {(Path('/usr/bin') / name).resolve(), (Path('/bin') / name).resolve()}
        require_guard(path.is_symlink() and path.resolve() in allowed, 'inert utility target changed: ' + name)
    for name in FAKE_COMMANDS:
        path = commands / name
        if name in absent:
            require_guard(not path.exists() and not path.is_symlink(), 'missing-command fixture still resolves')
        else:
            require_guard(path.is_file() and not path.is_symlink(), 'fake replaced by external command: ' + name)
            require_guard(path.read_text() == f'#!{sys.executable}\n' + FAKE, 'fake command changed: ' + name)
        # command -v performs name resolution only. No desktop command is invoked.
        result = subprocess.run([str(commands / 'sh'),'-c','command -v "$1"','preflight',name],env=env,
                                capture_output=True,text=True,timeout=10)
        if name in absent:
            require_guard(result.returncode != 0 and not result.stdout.strip(), 'absent command resolved: ' + name)
        else:
            require_guard(result.returncode == 0 and result.stdout.strip() == str(path), 'command escaped fake directory: ' + name)


def png_chunk(kind, data):
    return struct.pack('>I', len(data)) + kind + data + struct.pack('>I', zlib.crc32(kind + data))


PNG = b'\x89PNG\r\n\x1a\n' + png_chunk(b'IHDR', struct.pack('>IIBBBBB',1,1,8,2,0,0,0)) + png_chunk(b'IDAT',zlib.compress(b'\0\0\0\0')) + png_chunk(b'IEND',b'')
FAKE = r'''import json, os, pathlib, sys
name = pathlib.Path(sys.argv[0]).name
with open(os.environ['HOST_LOG'], 'a', encoding='utf-8') as stream:
    stream.write(json.dumps({'owner':os.environ['HOST_OWNER'],'name':name,'argv':sys.argv[1:]}, ensure_ascii=True) + '\n')
if name == 'tabula':
    if os.environ.get('HOST_FAIL'): sys.exit(9)
    args = dict(zip(sys.argv[1::2], sys.argv[2::2]))
    pathlib.Path(args['--state']).write_bytes(b'0' * (31 if os.environ.get('HOST_BAD_STATE') else 32))
    pathlib.Path(args['--output']).write_bytes(b'bad' if os.environ.get('HOST_BAD_PNG') else bytes.fromhex(os.environ['HOST_PNG']))
elif name == 'pmset':
    print("Now drawing from 'Battery Power'\n -InternalBattery-0 50%; discharging; 1:00 remaining" if not os.environ.get('HOST_INVALID') else 'unknown')
elif name == 'system_profiler':
    print('Resolution: 800 x 600' if not os.environ.get('HOST_INVALID') else 'Resolution: 999999999999 x invalid')
elif name == 'xrandr':
    print('HDMI connected primary 800x600+0+0' if not os.environ.get('HOST_INVALID') else 'unknown')
elif name == 'swaymsg':
    print('Current mode: 800x600 @ 60 Hz' if not os.environ.get('HOST_INVALID') else 'Current mode: invalid')
elif name in ('osascript','feh','swww'):
    image = pathlib.Path(sys.argv[-1])
    assert pathlib.Path(os.environ['HOST_ROOT']).resolve() in image.resolve().parents
    assert image.read_bytes().startswith(b'\x89PNG\r\n\x1a\n')
    assert len((image.parent/'mundus.hex').read_bytes()) == 32
elif name == 'plutil':
    import plistlib
    plistlib.loads(pathlib.Path(sys.argv[-1]).read_bytes())
'''


def execute(command, env, expected=0):
    result = subprocess.run(command, env=env, text=True, encoding='utf-8', errors='surrogateescape', capture_output=True, timeout=60)
    evidence = Path(env['HOST_ROOT']) / 'execution.jsonl'
    with evidence.open('a', encoding='utf-8') as stream:
        stream.write(json.dumps({'owner':env['HOST_OWNER'],'argv':command,'code':result.returncode,'stdout':result.stdout,'stderr':result.stderr},ensure_ascii=True) + '\n')
    assert result.returncode == expected, (command, result.returncode, result.stdout, result.stderr)
    return result.stdout


def entries(log):
    records = [json.loads(line) for line in log.read_text(encoding='utf-8-sig').splitlines()] if log.exists() else []
    owner = (log.parent / '.owner').read_text()
    for record in records:
        require_guard(isinstance(record,dict) and record.get('owner') == owner, 'unowned command record')
    return [(record['name'],record['argv']) for record in records]


def home_snapshot(home):
    """Record relative names, object kinds and file hashes without file contents."""
    snapshot = []
    for path in sorted(home.rglob('*')):
        require_guard(not path.is_symlink(), 'symbolic entry in observed HOME')
        relative = path.relative_to(home).as_posix()
        if path.is_dir():
            snapshot.append({'path':relative,'kind':'directory','sha256':None})
        else:
            require_guard(path.is_file(), 'non-regular entry in observed HOME')
            snapshot.append({'path':relative,'kind':'file','sha256':hashlib.sha256(path.read_bytes()).hexdigest()})
    return sorted(snapshot, key=lambda entry: entry['path'])


def read_before_snapshot(base, env):
    """Accept only the owner-tagged snapshot emitted before product invocation."""
    path = Path(env['HOST_BEFORE'])
    require_guard(path.resolve() == base.resolve() / 'before.json' and not path.is_symlink(),
                  'invalid pre-product snapshot path')
    document = json.loads(path.read_text(encoding='utf-8-sig'))
    require_guard(isinstance(document,dict) and document.get('owner') == env['HOST_OWNER'],
                  'unowned pre-product snapshot')
    records = document.get('entries')
    require_guard(isinstance(records,list), 'pre-product snapshot entries must be an array')
    names = set()
    for entry in records:
        require_guard(isinstance(entry,dict) and set(entry) == {'path','kind','sha256'},
                      'invalid pre-product snapshot record')
        name = entry['path']
        require_guard(isinstance(name,str) and name and not name.startswith('/') and
                      '\\' not in name and ':' not in name and all(part not in ('','.','..') for part in name.split('/')),
                      'snapshot name must be relative')
        require_guard(name not in names, 'duplicate pre-product snapshot entry')
        names.add(name)
        require_guard(entry['kind'] in ('directory','file'), 'invalid snapshot object kind')
        if entry['kind'] == 'directory':
            require_guard(entry['sha256'] is None, 'directory snapshot must not have a file hash')
        else:
            require_guard(isinstance(entry['sha256'],str) and re.fullmatch('[0-9a-f]{64}',entry['sha256']) is not None,
                          'invalid snapshot file hash')
    return sorted(records, key=lambda entry: entry['path'])


def verify_unix(system):
    with retained_workspace(system) as base:
        # Include every shell metacharacter required by the publication contract.
        home = base / "home ' & $ % ü"
        home.mkdir()
        hosts = base / "hosts ' & $ % ü"
        snapshot = reviewed_copy(hosts)
        fake_dir = base / 'commands'
        fake_dir.mkdir()
        # Only inert utilities are reachable. Missing applicators cannot fall through
        # to real desktop commands installed on the machine.
        for utility in INERT_UTILITIES:
            candidates = [Path('/usr/bin') / utility, Path('/bin') / utility]
            resolved = next((path for path in candidates if path.is_file()), None)
            require_guard(resolved is not None, 'missing inert system utility: ' + utility)
            (fake_dir / utility).symlink_to(resolved)
        log = base / 'commands.jsonl'
        for name in FAKE_COMMANDS:
            path = fake_dir / name
            path.write_text(f'#!{sys.executable}\n' + FAKE)
            path.chmod(0o755)
        binary = fake_dir / 'tabula'
        state = home / 'state " & $ % ü'
        env = isolated_environment(base, home, fake_dir)
        env['DISPLAY'] = ':fake'
        script = hosts / system / 'sisyphium.sh'
        if system == 'linux':
            # Fixture substitution is confined to a disposable script copy.
            supply = base / 'power_supply' / 'BAT0'
            supply.mkdir(parents=True)
            (supply / 'status').write_text('Discharging\n')
            (supply / 'capacity').write_text('50\n')
            script.write_text(script.read_text().replace('/sys/class/power_supply/*', shlex.quote(str(supply.parent)) + '/*'))
            snapshot[script] = hashlib.sha256(script.read_bytes()).hexdigest()
        def run(action=None, extra=(), expected=0, overrides=None):
            guard_snapshot(snapshot)
            actual_env = env | (overrides or {})
            absent = [name for name in FAKE_COMMANDS if not (fake_dir / name).exists()]
            preflight_unix(base, fake_dir, actual_env, absent)
            arguments = [str(fake_dir / 'sh'), str(script)] + ([action] if action else [])
            return execute(arguments + ['--binary',str(binary),'--state-dir',str(state)] + list(extra), actual_env, expected)
        def invocation(output):
            return dict(zip(shlex.split(output.removeprefix('command:'))[1::2], shlex.split(output.removeprefix('command:'))[2::2]))
        before = sorted(str(p) for p in home.rglob('*'))
        output = run()
        assert sorted(str(p) for p in home.rglob('*')) == before, sorted(str(p) for p in home.rglob('*'))
        assert not any(name in ('tabula','osascript','launchctl','systemctl','feh','swww') for name, _ in entries(log))
        measured = invocation(output)
        assert measured['--energy'] == '127' and measured['--width'] == '800' and measured['--height'] == '600', measured
        assert measured['--state'] == str(state / 'mundus.hex')
        if system == 'linux': (supply / 'capacity').write_text('101\n')
        fallback = invocation(run(overrides={'HOST_INVALID':'1'}))
        assert (fallback['--energy'],fallback['--width'],fallback['--height']) == ('255','320','180'), fallback
        explicit = invocation(run(extra=['--energy','001','--width','32','--height','18','--lingua','pt'],overrides={'HOST_INVALID':'1'}))
        assert (explicit['--energy'],explicit['--width'],explicit['--height'],explicit['--lingua']) == ('1','32','18','pt')
        for args in (['--interval','59'],['--energy','256'],['--width','0'],['--width','65535','--height','65535'],['--interval','999999999999999999'],['--lingua','xx'],['--wat','x'],['--energy','1','--energy','2']): run(extra=args, expected=2)
        original_state = state
        state = Path(os.fsdecode(os.fsencode(home) + b'/state-\xff'))
        # Some native filesystems reject arbitrary filename bytes. Preserve that
        # operational error rather than rewriting the caller's path lossily.
        try:
            state.mkdir()
        except OSError as error:
            if error.errno != errno.EILSEQ:
                raise
            run('run', extra=['--energy','255','--width','1','--height','1'], expected=1)
            assert not state.exists()
        else:
            run('run', extra=['--energy','255','--width','1','--height','1'])
            assert (state / 'tabula.png').read_bytes() == PNG
        state = original_state
        run('run')
        assert (state / 'mundus.hex').read_bytes() == b'0' * 32
        assert (state / 'tabula.png').read_bytes() == PNG
        assert not any(name in ('osascript','feh','swww') for name, _ in entries(log))
        run('apply')
        applicator = 'osascript' if system == 'macos' else 'feh'
        count = sum(name == applicator for name, _ in entries(log))
        assert count == 1
        for setting in ('HOST_FAIL','HOST_BAD_STATE','HOST_BAD_PNG'):
            run('apply', expected=1, overrides={setting:'1'})
            assert sum(name == applicator for name, _ in entries(log)) == count
        if system == 'linux':
            run('apply', extra=['--backend','swww'], overrides={'WAYLAND_DISPLAY':'wayland-fake'})
            assert any(name == 'swww' for name, _ in entries(log))
        # Missing dependencies fail before invoking tabula or creating new state.
        (fake_dir / applicator).unlink()
        preflight_unix(base, fake_dir, env, [applicator])
        prior_calls = entries(log)
        prior_files = {str(path):path.read_bytes() for path in home.rglob('*') if path.is_file()}
        run('apply', expected=1)
        new_calls = entries(log)[len(prior_calls):]
        assert not any(name in ('tabula',) + DANGEROUS_COMMANDS for name,_ in new_calls), new_calls
        assert {str(path):path.read_bytes() for path in home.rglob('*') if path.is_file()} == prior_files
        (fake_dir / applicator).write_text(f'#!{sys.executable}\n' + FAKE)
        (fake_dir / applicator).chmod(0o755)
        applied_before_install = sum(name in DANGEROUS_COMMANDS[:1] + ('feh','swww') for name,_ in entries(log))
        run('install',extra=['--energy','7','--width','32','--height','18','--lingua','pt'])
        assert sum(name in ('osascript','feh','swww') for name,_ in entries(log)) == applied_before_install
        if system == 'macos':
            schedule = home / 'Library/LaunchAgents/org.sisyphium.tabula.plist'
            document = plistlib.loads(schedule.read_bytes())
            assert document['StartInterval'] == 60 and 'RunAtLoad' not in document
            scheduled = document['ProgramArguments']
            assert scheduled[:3] == ['/bin/sh',str(script.resolve()),'apply']
            assert scheduled[scheduled.index('--state-dir')+1] == str(state)
            assert scheduled[scheduled.index('--lingua')+1] == 'pt'
            # plistlib parses structure without starting any native process.
            schedules = [schedule]
        else:
            unit_dir = home / 'config/systemd/user'
            service = (unit_dir / 'sisyphium.service').read_text()
            assert '$$' in service and '%%' in service and '\\"' in service and 'Type=oneshot' in service
            timer = (unit_dir / 'sisyphium.timer').read_text()
            assert 'OnUnitInactiveSec=60s' in timer and 'Persistent=' not in timer
            schedules = [unit_dir / 'sisyphium.service',unit_dir / 'sisyphium.timer']
            # Native systemd validation is an explicit, separate read-only gate.
        # Existing definitions are never silently overwritten.
        snapshots = {p:p.read_bytes() for p in schedules}
        run('install',expected=1)
        assert all(p.read_bytes() == data for p,data in snapshots.items())
        state_before = {p.name:p.read_bytes() for p in state.iterdir()}
        unrelated = schedules[0].parent / 'other-user-file'
        unrelated.write_text('preserve')
        run('uninstall')
        assert all(not p.exists() for p in schedules)
        assert unrelated.read_text() == 'preserve'
        assert {p.name:p.read_bytes() for p in state.iterdir()} == state_before
        run('uninstall')
        binary.unlink()
        run(expected=1)
        print(f'{system}: dry-run, measurements, precedence, bounds, run, apply gates, dependencies, schedule scope passed (fake commands)')


WINDOWS_WRAPPER = r'''
$ErrorActionPreference = 'Stop'
function Write-Call($Name, $Value) { Add-Content -LiteralPath $env:HOST_LOG -Encoding UTF8 -Value (@{owner=$env:HOST_OWNER;name=$Name;argv=@($Value)} | ConvertTo-Json -Compress -Depth 10) }
function Get-CimInstance { param($ClassName)
    if ($env:HOST_INVALID) { throw 'unavailable' }
    if ($ClassName -eq 'Win32_Battery') { [pscustomobject]@{BatteryStatus=1;EstimatedChargeRemaining=50} }
    else { [pscustomobject]@{CurrentHorizontalResolution=800;CurrentVerticalResolution=600} }
}
function Get-ScheduledTask { param($TaskName,$TaskPath,$ErrorAction)
    if (Test-Path -LiteralPath $env:HOST_TASK) { [pscustomobject]@{Description='sisyphium-owned'} }
}
function Register-ScheduledTask { param($TaskName,$TaskPath,$Xml)
    [IO.File]::WriteAllText($env:HOST_TASK,$Xml); Write-Call 'register' @($TaskName,$TaskPath)
}
function Stop-ScheduledTask { param($TaskName,$TaskPath) Write-Call 'stop' @($TaskName,$TaskPath) }
function Unregister-ScheduledTask { param($TaskName,$TaskPath,$Confirm)
    Remove-Item -LiteralPath $env:HOST_TASK; Write-Call 'unregister' @($TaskName,$TaskPath)
}
function Add-Type { param($TypeDefinition)
    Write-Call 'apply-binding' @()
    Microsoft.PowerShell.Utility\Add-Type -TypeDefinition 'public static class SisyphiumDesktop { public static string[] Arguments; public static bool SystemParametersInfoW(uint a,uint b,string p,uint f) { string root = System.IO.Path.GetFullPath(System.Environment.GetEnvironmentVariable("HOST_ROOT")) + System.IO.Path.DirectorySeparatorChar; if (!System.IO.Path.GetFullPath(p).StartsWith(root,System.StringComparison.OrdinalIgnoreCase)) throw new System.Exception("image escapes fake root"); Arguments = new string[] {a.ToString(), b.ToString(), p, f.ToString()}; return true; } }'
}
$overrides = @('Get-CimInstance','Get-ScheduledTask','Register-ScheduledTask','Stop-ScheduledTask','Unregister-ScheduledTask','Add-Type')
foreach ($name in $overrides) {
    $resolved = Get-Command -Name $name
    if ($resolved.CommandType -ne 'Function' -or $resolved.ScriptBlock.File -ne $PSCommandPath) { throw "untrusted command override: $name" }
}
$tokens = $null; $errors = $null
$tree = [Management.Automation.Language.Parser]::ParseFile($env:HOST_SCRIPT,[ref]$tokens,[ref]$errors)
if ($errors.Count -ne 0) { throw 'product parse errors' }
$commands = $tree.FindAll({param($node) $node -is [Management.Automation.Language.CommandAst]},$true)
foreach ($node in $commands) {
    $name = $node.GetCommandName()
    if ($null -eq $name) {
        if ($node.Extent.Text -ne '& $Binary @arguments') { throw 'unreviewed dynamic command' }
    } elseif ($name -match '[\\/:]' -or $name -in @('Invoke-Expression','Start-Process','schtasks','rundll32','cmd','powershell','pwsh')) {
        throw "unreviewed external command: $name"
    }
}
$values = [string[]](ConvertFrom-Json -InputObject (Get-Content -LiteralPath $env:HOST_ARGS -Raw))
# The interpreter and command preflight run before the measured product boundary.
# Keep the full before snapshot private; do not print file contents or argv.
$homeRoot = [IO.Path]::GetFullPath($env:HOME).TrimEnd([IO.Path]::DirectorySeparatorChar) + [IO.Path]::DirectorySeparatorChar
$beforeEntries = @()
$pending = New-Object 'System.Collections.Generic.Stack[string]'
$pending.Push($homeRoot)
$hasher = [Security.Cryptography.SHA256]::Create()
try {
    while ($pending.Count -gt 0) {
        foreach ($entryPath in [IO.Directory]::GetFileSystemEntries($pending.Pop())) {
            $attributes = [IO.File]::GetAttributes($entryPath)
            if ($attributes -band [IO.FileAttributes]::ReparsePoint) { throw 'symbolic entry in observed HOME' }
            $relative = $entryPath.Substring($homeRoot.Length).Replace('\','/')
            if ($attributes -band [IO.FileAttributes]::Directory) {
                $beforeEntries += [pscustomobject]@{path=$relative;kind='directory';sha256=$null}
                $pending.Push($entryPath)
            } else {
                $stream = [IO.File]::OpenRead($entryPath)
                try { $digest = [BitConverter]::ToString($hasher.ComputeHash($stream)).Replace('-','').ToLowerInvariant() }
                finally { $stream.Dispose() }
                $beforeEntries += [pscustomobject]@{path=$relative;kind='file';sha256=$digest}
            }
        }
    }
} finally { $hasher.Dispose() }
$beforeJson = ConvertTo-Json -InputObject @{owner=$env:HOST_OWNER;entries=@($beforeEntries)} -Compress -Depth 8
[IO.File]::WriteAllText($env:HOST_BEFORE,$beforeJson)
& $env:HOST_SCRIPT @values
$code = $LASTEXITCODE
if ('SisyphiumDesktop' -as [type]) {
    if ($null -ne [SisyphiumDesktop]::Arguments) { Write-Call 'apply' ([SisyphiumDesktop]::Arguments) }
}
exit $code
'''
WINDOWS_BINARY = r'''
if ($env:HOST_FAIL) { exit 9 }
$values = @{}; for($i=0;$i -lt $args.Count;$i+=2) {$values[$args[$i]]=$args[$i+1]}
Add-Content -LiteralPath $env:HOST_LOG -Encoding UTF8 -Value (@{owner=$env:HOST_OWNER;name='tabula';argv=@($args)} | ConvertTo-Json -Compress -Depth 5)
$length=32; if ($env:HOST_BAD_STATE) {$length=31}
[IO.File]::WriteAllText($values['--state'],('0' * $length))
[byte[]]$png=@PNG_BYTES@; if ($env:HOST_BAD_PNG) {$png=1,2,3}
[IO.File]::WriteAllBytes($values['--output'],$png)
exit 0
'''


def verify_windows():
    shell = shutil.which('powershell') or shutil.which('pwsh')
    assert shell, 'PowerShell is required for Windows verification'
    with retained_workspace('windows') as base:
        home = base / "home ' & $ % ü"
        home.mkdir()
        fake_dir = base / 'commands'
        fake_dir.mkdir()
        hosts = base / 'hosts'
        snapshot = reviewed_copy(hosts)
        binary = base / "tabula ' & $ ü.ps1"
        binary.write_text(WINDOWS_BINARY.replace('@PNG_BYTES@',','.join(str(byte) for byte in PNG)),encoding='utf-8-sig')
        wrapper = base / 'wrapper.ps1'
        wrapper.write_text(WINDOWS_WRAPPER,encoding='utf-8-sig')
        state = home / 'state & $ ü'
        log = base / 'calls.jsonl'; task = base / 'task.xml'; arguments = base / 'arguments.json'
        env = isolated_environment(base, home, fake_dir)
        env['PSModulePath'] = str(Path(shell).parent / 'Modules')
        env.update(HOST_SCRIPT=str(hosts/'windows/sisyphium.ps1'), HOST_ARGS=str(arguments), HOST_LOG=str(log), HOST_TASK=str(task))
        wrapper_hash = hashlib.sha256(wrapper.read_bytes()).hexdigest()
        def run(action='dry-run', extra=(), expected=0, overrides=None):
            guard_snapshot(snapshot)
            actual_env = env | (overrides or {})
            guard_environment(base, actual_env, fake_dir)
            require_guard(hashlib.sha256(wrapper.read_bytes()).hexdigest() == wrapper_hash, 'Windows wrapper changed')
            if binary.exists():
                require_guard(binary.read_text(encoding='utf-8-sig') == WINDOWS_BINARY.replace('@PNG_BYTES@',','.join(str(byte) for byte in PNG)), 'Windows fake binary changed')
            arguments.write_text(json.dumps(['-Action',action,'-Binary',str(binary),'-StateDir',str(state)] + list(extra)),encoding='utf-8-sig')
            return execute([shell,'-NoProfile','-NonInteractive','-ExecutionPolicy','Bypass','-File',str(wrapper)],actual_env,expected)
        output = run()
        before = read_before_snapshot(base, env)
        print('windows: pre-product HOME entries: ' + json.dumps([entry['path'] for entry in before], ensure_ascii=True))
        after = home_snapshot(home)
        fake_calls = [name for name, _ in entries(log)]
        task_exists = task.exists()
        # Equality includes kinds and hashes: existing files cannot change, and no
        # new entry is allowed. Diagnostics expose relative fixture names only.
        assert before == after, {
            'dry_run_home_before': [entry['path'] for entry in before],
            'dry_run_home_after': [entry['path'] for entry in after],
            'dry_run_home_changed': sorted({entry['path'] for entry in before + after
                                           if entry not in before or entry not in after}),
        }
        assert not fake_calls, {'dry_run_fake_calls': fake_calls}
        assert not task_exists, {'dry_run_task_exists': task_exists}
        assert '"127"' in output and '"800"' in output and '"600"' in output
        output = run(overrides={'HOST_INVALID':'1'})
        assert '"255"' in output and '"320"' in output and '"180"' in output
        run(extra=['-Interval','59'],expected=2)
        run(extra=['-Width','65535','-Height','65535'],expected=2)
        run(extra=['-Energy','256'],expected=2)
        run(extra=['-Lingua','xx'],expected=2)
        run('run',extra=['-Energy','7','-Width','32','-Height','18','-Lingua','pt'])
        called = dict(zip(entries(log)[-1][1][::2],entries(log)[-1][1][1::2]))
        assert called['--energy'] == '7' and called['--state'] == str(state/'mundus.hex') and called['--lingua'] == 'pt'
        assert (state/'mundus.hex').read_bytes() == b'0'*32 and (state/'tabula.png').read_bytes() == PNG
        run('apply')
        count = sum(name == 'apply' for name,_ in entries(log)); assert count == 1
        for setting in ('HOST_FAIL','HOST_BAD_STATE','HOST_BAD_PNG'):
            run('apply',expected=1,overrides={setting:'1'})
            assert sum(name == 'apply' for name,_ in entries(log)) == count
        count_before_install = sum(name == 'apply' for name,_ in entries(log))
        run('install',extra=['-Lingua','pt'])
        assert sum(name == 'apply' for name,_ in entries(log)) == count_before_install
        xml = ET.fromstring(task.read_bytes()); ns = {'t':'http://schemas.microsoft.com/windows/2004/02/mit/task'}
        assert xml.find('.//t:MultipleInstancesPolicy',ns).text == 'IgnoreNew'
        assert xml.find('.//t:Interval',ns).text == 'PT60S'
        assert str(state) in xml.find('.//t:Arguments',ns).text
        run('install',expected=1)
        before = {p.name:p.read_bytes() for p in state.iterdir()}
        run('uninstall')
        assert not task.exists() and {p.name:p.read_bytes() for p in state.iterdir()} == before
        run('uninstall')
        binary.unlink(); run(expected=1)
        print('windows: dry-run, measurements, precedence, bounds, publication gates and schedule scope passed (fake commands)')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--platform', choices=('current','macos','linux','windows','unix'),default='current')
    selected = parser.parse_args().platform
    if selected == 'current': selected = {'Darwin':'macos','Linux':'linux','Windows':'windows'}[platform.system()]
    if selected == 'windows': verify_windows()
    elif selected == 'unix':
        verify_unix('macos'); verify_unix('linux')
    else: verify_unix(selected)


if __name__ == '__main__':
    main()
