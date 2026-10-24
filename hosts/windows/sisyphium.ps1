# Native host contracts: hosts/README.md and docs/referencias.md.
$ErrorActionPreference = 'Stop'
# Parse explicitly so every argument error has exit status 2 on PowerShell 5.1.
function Stop-Usage([string]$Message) { [Console]::Error.WriteLine("sisyphium: $Message"); exit 2 }
function Read-Number([string]$Text, [long]$Minimum, [long]$Maximum) {
    $number = 0L
    if ($Text -notmatch '^[0-9]+$' -or -not [long]::TryParse($Text, [ref]$number) -or $number -lt $Minimum -or $number -gt $Maximum) { Stop-Usage "integer outside $Minimum..$Maximum" }
    return $number
}
# Windows command-line quoting doubles slashes preceding a quote or closing quote.
function Quote-Argument([string]$Text) {
    return '"' + [regex]::Replace([regex]::Replace($Text, '(\\*)"', '$1$1\"'), '(\\+)$', '$1$1') + '"'
}
$Action = 'dry-run'; $Binary = Join-Path $PSScriptRoot '../../target/release/sisyphium-tabula.exe'
$StateDir = Join-Path $env:LOCALAPPDATA 'sisyphium'
$Interval = 60L; $Energy = $null; $Width = $null; $Height = $null; $Lingua = 'en'
$seen = @{}; $tokens = @($args)
for ($i = 0; $i -lt $tokens.Count; $i += 2) {
    $key = [string]$tokens[$i]
    if ($seen.ContainsKey($key) -or $i + 1 -ge $tokens.Count) { Stop-Usage "duplicate or incomplete option: $key" }
    $seen[$key] = $true; $value = [string]$tokens[$i + 1]
    switch ($key) {
        '-Action' { if ($value -notin @('dry-run','run','apply','install','uninstall')) { Stop-Usage 'invalid action' }; $Action = $value }
        '-Binary' { $Binary = $value }
        '-StateDir' { $StateDir = $value }
        '-Interval' { $Interval = Read-Number $value 60 2147483647 }
        '-Energy' { $Energy = Read-Number $value 0 255 }
        '-Width' { $Width = Read-Number $value 1 65535 }
        '-Height' { $Height = Read-Number $value 1 65535 }
        '-Lingua' { if ($value -notin @('en','pt')) { Stop-Usage 'lingua must be en or pt' }; $Lingua = $value }
        default { Stop-Usage "unknown option: $key" }
    }
}
if ([string]::IsNullOrWhiteSpace($Binary) -or [string]::IsNullOrWhiteSpace($StateDir)) { Stop-Usage 'paths must not be empty' }
$taskName = 'Sisyphium.Tabula'; $taskPath = '\'; $owner = 'sisyphium-owned'
try {
    $Binary = [IO.Path]::GetFullPath($Binary); $StateDir = [IO.Path]::GetFullPath($StateDir)
    if ($Action -eq 'uninstall') {
        $task = Get-ScheduledTask -TaskName $taskName -TaskPath $taskPath -ErrorAction SilentlyContinue
        if ($null -ne $task) {
            if ($task.Description -ne $owner) { throw 'unrecognized scheduled task' }
            Stop-ScheduledTask -TaskName $taskName -TaskPath $taskPath
            Unregister-ScheduledTask -TaskName $taskName -TaskPath $taskPath -Confirm:$false
        }
        exit 0
    }
    if (-not (Test-Path -LiteralPath $Binary -PathType Leaf)) { throw "missing executable: $Binary" }
    if ($Action -eq 'install') {
        # Task Scheduler accepts repetition durations of 1 minute through 31 days.
        if ($Interval -gt 2678400) { Stop-Usage 'Task Scheduler interval exceeds 31 days' }
        if (Get-ScheduledTask -TaskName $taskName -TaskPath $taskPath -ErrorAction SilentlyContinue) { throw 'task exists; uninstall first' }
        $arguments = @('-NoProfile','-NonInteractive','-File',$PSCommandPath,'-Action','apply','-Binary',$Binary,'-StateDir',$StateDir,'-Interval',"$Interval",'-Lingua',$Lingua)
        if ($null -ne $Energy) { $arguments += @('-Energy',"$Energy") }
        if ($null -ne $Width) { $arguments += @('-Width',"$Width") }
        if ($null -ne $Height) { $arguments += @('-Height',"$Height") }
        $command = Join-Path $PSHOME 'powershell.exe'
        if (-not (Test-Path -LiteralPath $command)) { $command = Join-Path $PSHOME 'pwsh.exe' }
        $xml = [xml](Get-Content -LiteralPath (Join-Path $PSScriptRoot 'sisyphium-task.xml.in') -Raw)
        $xml.Task.Triggers.TimeTrigger.StartBoundary = (Get-Date).AddSeconds($Interval).ToString('s')
        $xml.Task.Triggers.TimeTrigger.Repetition.Interval = "PT${Interval}S"
        $xml.Task.Actions.Exec.Command = $command.ToString()
        $xml.Task.Actions.Exec.Arguments = ($arguments | ForEach-Object { Quote-Argument $_ }) -join ' '
        $xml.Task.Principals.Principal.UserId = [Security.Principal.WindowsIdentity]::GetCurrent().User.Value
        Register-ScheduledTask -TaskName $taskName -TaskPath $taskPath -Xml $xml.OuterXml | Out-Null
        exit 0
    }
    if ($null -eq $Energy) {
        $Energy = 255L
        try {
            $battery = Get-CimInstance -ClassName Win32_Battery | Select-Object -First 1
            # BatteryStatus=1 is discharging; unknown or AC retains 255.
            if ($null -ne $battery -and $battery.BatteryStatus -eq 1 -and "$($battery.EstimatedChargeRemaining)" -match '^[0-9]+$' -and $battery.EstimatedChargeRemaining -le 100) {
                $Energy = [long][Math]::Floor([double]$battery.EstimatedChargeRemaining * 255 / 100)
            }
        } catch { $Energy = 255L }
    }
    $measuredWidth = 320L; $measuredHeight = 180L
    try {
        $screen = Get-CimInstance -ClassName Win32_VideoController | Select-Object -First 1
        $w = [long]$screen.CurrentHorizontalResolution; $h = [long]$screen.CurrentVerticalResolution
        if ($w -ge 1 -and $w -le 65535 -and $h -ge 1 -and $h -le 65535 -and $w * $h -le 33554432) { $measuredWidth = $w; $measuredHeight = $h }
    } catch { }
    if ($null -eq $Width) { $Width = $measuredWidth }; if ($null -eq $Height) { $Height = $measuredHeight }
    if ($Width * $Height -gt 33554432) { Stop-Usage 'image area exceeds 33554432 pixels' }
    $state = Join-Path $StateDir 'mundus.hex'; $image = Join-Path $StateDir 'tabula.png'
    $arguments = @('--state',$state,'--output',$image,'--format','png','--energy',"$Energy",'--width',"$Width",'--height',"$Height",'--lingua',$Lingua)
    Write-Output ('command: ' + ((@($Binary) + $arguments | ForEach-Object { Quote-Argument $_ }) -join ' '))
    if ($Action -eq 'dry-run') { exit 0 }
    [IO.Directory]::CreateDirectory($StateDir) | Out-Null
    $global:LASTEXITCODE = 0
    & $Binary @arguments
    if (-not $? -or $LASTEXITCODE -ne 0) { throw 'tabula failed; wallpaper was not applied' }
    foreach ($file in @($state,$image)) {
        if ((Get-Item -LiteralPath $file).Attributes -band [IO.FileAttributes]::ReparsePoint) { throw 'symbolic publication is not supported' }
    }
    $bytes = [IO.File]::ReadAllBytes($state)
    if ($bytes.Length -ne 32 -or [Text.Encoding]::ASCII.GetString($bytes) -notmatch '\A[0-9a-fA-F]{32}\z') { throw 'invalid state' }
    $stream = [IO.File]::OpenRead($image)
    try { $signature = New-Object byte[] 8; $count = $stream.Read($signature,0,8) } finally { $stream.Dispose() }
    if ($count -ne 8 -or [BitConverter]::ToString($signature) -ne '89-50-4E-47-0D-0A-1A-0A') { throw 'invalid PNG signature' }
    if ($Action -eq 'apply') {
        # SPI_SETDESKWALLPAPER=20; update profile=1 and broadcast change=2.
        Add-Type -TypeDefinition @'
using System;
using System.Runtime.InteropServices;
public static class SisyphiumDesktop {
    [DllImport("user32.dll", CharSet=CharSet.Unicode, SetLastError=true)]
    [return: MarshalAs(UnmanagedType.Bool)]
    public static extern bool SystemParametersInfoW(uint action, uint parameter, string path, uint flags);
}
'@
        if (-not [SisyphiumDesktop]::SystemParametersInfoW(20,0,$image,3)) { throw "SystemParametersInfoW failed: $([Runtime.InteropServices.Marshal]::GetLastWin32Error())" }
    }
    exit 0
} catch { [Console]::Error.WriteLine("sisyphium: $($_.Exception.Message)"); exit 1 }
