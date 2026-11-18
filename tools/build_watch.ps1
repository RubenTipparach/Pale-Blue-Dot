# Build pbd-app and say plainly when the build is stuck, and why
# (build-play.bat calls this). Windows PowerShell 5.1.
#
# Before the build:
#   - another cargo, rustc or rust-analyzer running (they hold target/ or the
#     package cache, and this build waits on their lock without a word);
#   - the game still running from the exe being built (Windows will not let
#     the link replace a running exe: "Access is denied");
#   - less than 10 GB free on the drive.
# During the build, every 20 s with no new output: what cargo is doing (a
# lock, the network, compiling, linking), from its last line and the processes
# running under it.
param(
    [string]$Profile = "fast"
)

$ErrorActionPreference = "Continue"
$root = Split-Path -Parent $PSScriptRoot
Set-Location $root

# QuickEdit off for this console window (owner, 2026-11-17: the build "was
# stuck on something random", and Esc set it going): with QuickEdit on, a
# click in the window starts a selection and freezes every program writing to
# it, the build and this watchdog alike, until Esc or Enter. The console is
# shared with build-play.bat, so the game's output stays live too.
try {
    Add-Type -Namespace PbdConsole -Name Mode -MemberDefinition @'
[DllImport("kernel32.dll")] public static extern System.IntPtr GetStdHandle(int which);
[DllImport("kernel32.dll")] public static extern bool GetConsoleMode(System.IntPtr h, out uint mode);
[DllImport("kernel32.dll")] public static extern bool SetConsoleMode(System.IntPtr h, uint mode);
'@
    $stdin = [PbdConsole.Mode]::GetStdHandle(-10)
    $mode = 0
    if ([PbdConsole.Mode]::GetConsoleMode($stdin, [ref]$mode)) {
        # ENABLE_EXTENDED_FLAGS on, ENABLE_QUICK_EDIT_MODE off.
        [void][PbdConsole.Mode]::SetConsoleMode($stdin, ($mode -bor 0x80) -band (-bnot 0x40))
    }
} catch {
    Write-Host "(QuickEdit could not be turned off: a click in this window pauses the build until Esc.)"
}

function Show-Others {
    $others = Get-CimInstance Win32_Process |
        Where-Object { $_.Name -match '^(cargo|rustc|rust-analyzer|cargo-clippy|clippy-driver)\.exe$' -and $_.ProcessId -ne $PID }
    foreach ($p in $others) {
        $cmd = if ($p.CommandLine) { $p.CommandLine } else { "" }
        if ($cmd.Length -gt 140) { $cmd = $cmd.Substring(0, 140) + "..." }
        Write-Host ("    {0,-18} pid {1,-6} {2}" -f $p.Name, $p.ProcessId, $cmd)
    }
    return @($others).Count
}

# --- Pre-flight ---------------------------------------------------------------
$exe = Join-Path $root "target\$Profile\pbd-app.exe"
$running = Get-CimInstance Win32_Process | Where-Object { $_.Name -eq 'pbd-app.exe' -and $_.ExecutablePath -eq $exe }
if ($running) {
    Write-Host ""
    Write-Host "!! The game is still running from target\$Profile\pbd-app.exe (pid $($running.ProcessId -join ', '))." -ForegroundColor Yellow
    Write-Host "   Windows cannot replace a running exe, so the build would fail at the link." -ForegroundColor Yellow
    Write-Host "   Close the game, then press a key to go on." -ForegroundColor Yellow
    [void][System.Console]::ReadKey($true)
}

Write-Host "Checking for other Rust tools running..."
$n = Show-Others
if ($n -gt 0) {
    Write-Host "!! $n Rust tool(s) above are running. If one of them is a build or rust-analyzer, this build will" -ForegroundColor Yellow
    Write-Host "   wait for its lock ('Blocking waiting for file lock'). It continues on its own when they finish." -ForegroundColor Yellow
} else {
    Write-Host "   none."
}

$drive = (Get-Item $root).PSDrive
$freeGb = [math]::Round($drive.Free / 1GB, 1)
if ($freeGb -lt 10) {
    Write-Host "!! Only $freeGb GB free on $($drive.Name):. A release build can need several GB; clear target\ if it fails." -ForegroundColor Yellow
}

# --- The build, watched -------------------------------------------------------
$args = @("build", "--profile", $Profile, "--locked", "-p", "pbd-app")
Write-Host ""
Write-Host "cargo $($args -join ' ')"
$psi = New-Object System.Diagnostics.ProcessStartInfo
$psi.FileName = "cargo"
$psi.Arguments = $args -join ' '
$psi.UseShellExecute = $false
$psi.RedirectStandardError = $true
$psi.RedirectStandardOutput = $true
$psi.WorkingDirectory = $root
$proc = New-Object System.Diagnostics.Process
$proc.StartInfo = $psi

$script:last = ""
$script:lastAt = Get-Date
$queue = [System.Collections.Concurrent.ConcurrentQueue[string]]::new()
$handler = { if ($EventArgs.Data -ne $null) { $Event.MessageData.Enqueue($EventArgs.Data) } }
$e1 = Register-ObjectEvent -InputObject $proc -EventName ErrorDataReceived -Action $handler -MessageData $queue
$e2 = Register-ObjectEvent -InputObject $proc -EventName OutputDataReceived -Action $handler -MessageData $queue
[void]$proc.Start()
$proc.BeginErrorReadLine()
$proc.BeginOutputReadLine()
$started = Get-Date
$lastReport = Get-Date

while (-not $proc.HasExited -or -not $queue.IsEmpty) {
    $line = $null
    while ($queue.TryDequeue([ref]$line)) {
        Write-Host $line
        $script:last = $line
        $script:lastAt = Get-Date
        if ($line -match 'Blocking waiting for file lock') {
            Write-Host "!! Waiting on another tool's lock. Holding it:" -ForegroundColor Yellow
            [void](Show-Others)
        }
    }
    $quiet = ((Get-Date) - $script:lastAt).TotalSeconds
    if ($quiet -gt 20 -and ((Get-Date) - $lastReport).TotalSeconds -gt 20) {
        $lastReport = Get-Date
        # Every process under the build: `cargo` here is rustup's proxy, so
        # the real cargo and its rustc and linker are grandchildren and lower.
        $all = Get-CimInstance Win32_Process
        $ids = @($proc.Id)
        $kids = @()
        do {
            $next = @($all | Where-Object { $ids -contains $_.ParentProcessId -and -not ($kids.ProcessId -contains $_.ProcessId) })
            $kids += $next
            $ids = @($next.ProcessId)
        } while ($next.Count -gt 0)
        $kids = @($kids | Where-Object { $_.Name -ne 'cargo.exe' -and $_.Name -ne 'conhost.exe' })
        $names = ($kids | ForEach-Object { $_.Name }) -join ', '
        $mins = [math]::Round(((Get-Date) - $started).TotalMinutes, 1)
        $why = if ($script:last -match 'Blocking waiting') { "waiting on another tool's lock (see above)" }
            elseif ($script:last -match 'Updating|Downloading|index') { "on the network (crates.io); check the connection, or run cargo with --offline" }
            elseif ($kids | Where-Object { $_.Name -match 'link|lld' }) { "linking" + $(if ($Profile -eq 'release') { " with link-time optimisation, which takes minutes" } else { "" }) }
            elseif ($kids | Where-Object { $_.Name -eq 'rustc.exe' }) { "compiling ($(@($kids | Where-Object { $_.Name -eq 'rustc.exe' }).Count) rustc)" + $(if ($Profile -eq 'release') { "; release optimises the whole crate at once, so pbd-app alone can take several minutes" } else { "" }) }
            elseif (-not $kids) { "cargo is running nothing: likely waiting on a lock or the network" }
            else { "running $names" }
        Write-Host ("   ... {0} min in, quiet {1:N0} s: {2}" -f $mins, $quiet, $why) -ForegroundColor Cyan
    }
    Start-Sleep -Milliseconds 250
}
# WaitForExit() with no timeout waits for the output readers to finish too;
# then whatever they queued after the loop's last look is printed.
$proc.WaitForExit()
Start-Sleep -Milliseconds 300
$line = $null
while ($queue.TryDequeue([ref]$line)) {
    Write-Host $line
    $script:last = $line
}
Unregister-Event -SourceIdentifier $e1.Name
Unregister-Event -SourceIdentifier $e2.Name
$code = $proc.ExitCode
if ($code -ne 0 -and $script:last -match 'Access is denied|being used by another process') {
    Write-Host "!! The exe is locked: the game (or a debugger) still has it open. Close it and build again." -ForegroundColor Yellow
}
exit $code
