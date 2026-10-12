#requires -Version 5.1
<#
.SYNOPSIS
Runs sequential release captures and records wall-frame timing and process memory.
.EXAMPLE
powershell -NoProfile -ExecutionPolicy Bypass -File tools/benchmark.ps1
.EXAMPLE
./tools/benchmark.ps1 -Scenes surface,polar -Rounds 1 -Executable ./target/release/pbd-app.exe
#>
[CmdletBinding()]
param(
    [string]$Executable = '',
    [string]$OutputBase = '',
    [ValidateSet('walk', 'surface', 'orbit', 'coast', 'night', 'pole', 'tour', 'polar')]
    [string[]]$Scenes = @('walk', 'surface', 'orbit', 'tour', 'polar'),
    [ValidateRange(1, 100)][int]$Rounds = 3,
    [ValidateRange(120, 100000)][int]$StaticFrames = 1200,
    [ValidateRange(3600, 100000)][int]$TourFrames = 3600,
    [ValidateRange(5, 600)][int]$TimeoutSeconds = 60,
    [ValidateRange(1.0, 1000.0)][double]$TargetFrameMs = (1000.0 / 60.0),
    [switch]$KeepFinalFocus
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$repoRoot = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
if ([string]::IsNullOrWhiteSpace($Executable)) {
    $Executable = Join-Path $repoRoot 'target/release/pbd-app.exe'
}
if ([string]::IsNullOrWhiteSpace($OutputBase)) {
    $OutputBase = Join-Path $repoRoot 'output/captures/performance'
}
$exePath = (Resolve-Path -LiteralPath $Executable).ProviderPath
if (-not (Test-Path -LiteralPath $exePath -PathType Leaf)) {
    throw "Build the release executable first: cargo build --release --locked -p pbd-app"
}

function Assert-NoOtherGame {
    $otherGames = @(Get-Process -Name 'pbd-app' -ErrorAction SilentlyContinue)
    if ($otherGames.Count -gt 0) {
        throw "Close the existing pbd-app processes before benchmarking (PIDs: $($otherGames.Id -join ', ')). The harness will not close them."
    }
}

function ConvertTo-NativeArgument([string]$Argument) {
    # Windows CommandLineToArgvW quoting, including trailing backslashes.
    '"' + (($Argument -replace '(\\*)"', '$1$1\"') -replace '(\\+)$', '$1$1') + '"'
}

function ConvertTo-InvariantDouble([string]$Value) {
    [double]::Parse($Value, [Globalization.CultureInfo]::InvariantCulture)
}

Assert-NoOtherGame
if (-not ('PbdBenchmark.Windows' -as [type])) {
    Add-Type -TypeDefinition @'
using System;
using System.Runtime.InteropServices;
using System.Text;
namespace PbdBenchmark {
    public static class Windows {
        private delegate bool EnumWindowsProc(IntPtr hwnd, IntPtr data);
        [DllImport("user32.dll")] private static extern bool EnumWindows(EnumWindowsProc callback, IntPtr data);
        [DllImport("user32.dll", CharSet = CharSet.Unicode)] private static extern int GetWindowText(IntPtr hwnd, StringBuilder text, int count);
        [DllImport("user32.dll")] private static extern bool IsWindowVisible(IntPtr hwnd);
        [DllImport("user32.dll")] public static extern bool IsWindow(IntPtr hwnd);
        [DllImport("user32.dll")] public static extern uint GetWindowThreadProcessId(IntPtr hwnd, out uint pid);
        [DllImport("user32.dll")] public static extern IntPtr GetForegroundWindow();
        [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr hwnd);
        public static IntPtr FindExplorerWindow(int processId) {
            IntPtr found = IntPtr.Zero;
            EnumWindows(delegate(IntPtr hwnd, IntPtr data) {
                uint owner;
                GetWindowThreadProcessId(hwnd, out owner);
                if (owner != processId || !IsWindowVisible(hwnd)) return true;
                var title = new StringBuilder(256);
                GetWindowText(hwnd, title, title.Capacity);
                if (title.ToString() != "Pale Blue Dot | Planet Explorer") return true;
                found = hwnd;
                return false;
            }, IntPtr.Zero);
            return found;
        }
    }
}
'@
}

$initialForeground = [PbdBenchmark.Windows]::GetForegroundWindow()
[uint32]$initialForegroundPid = 0
$null = [PbdBenchmark.Windows]::GetWindowThreadProcessId($initialForeground, [ref]$initialForegroundPid)
$runId = (Get-Date -Format 'yyyyMMdd-HHmmss-fff') + '-' + [guid]::NewGuid().ToString('N').Substring(0, 8)
$outputDir = Join-Path ([IO.Path]::GetFullPath($OutputBase)) $runId
$null = New-Item -ItemType Directory -Path $outputDir
$resultsPath = Join-Path $outputDir 'results.json'
$csvPath = Join-Path $outputDir 'summary.csv'
$results = New-Object 'System.Collections.Generic.List[object]'
$exeInfo = Get-Item -LiteralPath $exePath
$hardware = [ordered]@{ cpu = @(); gpu = @(); os = $null; installed_memory_mib = $null }
try {
    $hardware.cpu = @(Get-CimInstance Win32_Processor | Select-Object -ExpandProperty Name)
    $hardware.gpu = @(Get-CimInstance Win32_VideoController | Select-Object Name, DriverVersion)
    $hardware.os = (Get-CimInstance Win32_OperatingSystem).Caption
    $hardware.installed_memory_mib = [math]::Round((Get-CimInstance Win32_ComputerSystem).TotalPhysicalMemory / 1MB, 1)
} catch {
    Write-Warning "Some hardware metadata could not be read: $($_.Exception.Message)"
}
$shaderHashes = @(Get-ChildItem -LiteralPath (Join-Path $repoRoot 'assets/shaders') -Filter '*.wgsl' -Recurse |
    ForEach-Object {
        [ordered]@{ path = $_.FullName.Substring($repoRoot.Length + 1); sha256 = (Get-FileHash -LiteralPath $_.FullName -Algorithm SHA256).Hash }
    })
$metadata = [ordered]@{
    schema_version = 1
    started_utc = [DateTime]::UtcNow.ToString('o')
    executable = $exePath
    executable_sha256 = (Get-FileHash -LiteralPath $exePath -Algorithm SHA256).Hash
    executable_modified_utc = $exeInfo.LastWriteTimeUtc.ToString('o')
    executable_bytes = $exeInfo.Length
    shader_hashes = $shaderHashes
    hardware = $hardware
    resolution = '1440x900 (current application default; check capture dimensions)'
    terrain_seed = '0x5eed2026 (current application default)'
    warmup_frames = 60
    screenshot_frames_excluded = $true
    timing = 'Application-reported wall-frame intervals; not GPU timestamps. Capture mode requests AutoNoVsync; interactive mode uses VSync with a one-frame latency hint.'
    memory = 'Process memory sampled about every 100 ms across startup and rendering. Peak working set also uses the OS process counter. These are CPU process figures, not GPU VRAM.'
    terrain_upload = 'Current renderer: one immutable 128-byte record per column (655362 columns), 112-byte view/time update per view, two GPU-resident 2.5 MiB ID lists (terrain and foliage), and 32 bytes of indirect arguments per view. These are implementation sizes, not measured transfer counters.'
    target_frame_ms = $TargetFrameMs
    static_frames = $StaticFrames
    tour_frames = $TourFrames
    timeout_seconds = $TimeoutSeconds
    rounds = $Rounds
    scenes = $Scenes
}

function Save-Results {
    [ordered]@{ metadata = $metadata; runs = @($results.ToArray()) } |
        ConvertTo-Json -Depth 8 | Set-Content -LiteralPath $resultsPath -Encoding UTF8
    if ($results.Count -gt 0) {
        $results | Export-Csv -LiteralPath $csvPath -NoTypeInformation -Encoding UTF8
    }
}

$child = $null
Save-Results
Write-Host "Benchmark output: $outputDir"
Write-Host 'Keep the benchmark game window visible and avoid interacting with other windows during each run.'
try {
    for ($round = 1; $round -le $Rounds; $round++) {
        foreach ($scene in $Scenes) {
            Assert-NoOtherGame
            $stem = '{0}-round-{1:D2}' -f $scene, $round
            $capturePath = Join-Path $outputDir ($stem + '.png')
            $stdoutPath = Join-Path $outputDir ($stem + '.stdout.txt')
            $stderrPath = Join-Path $outputDir ($stem + '.stderr.txt')
            $isTour = $scene -eq 'tour' -or $scene -eq 'polar'
            $frames = if ($isTour) { $TourFrames } else { $StaticFrames }
            $view = if ($scene -eq 'polar') { 'pole' } elseif ($scene -eq 'tour' -or $scene -eq 'walk') { 'coast' } else { $scene }
            $arguments = @('--capture', $capturePath, '--view', $view, '--frames', "$frames", '--fixed-dt')
            if ($isTour) { $arguments += '--tour' }
            if ($scene -eq 'walk') { $arguments += '--walk' }
            $argumentLine = ($arguments | ForEach-Object { ConvertTo-NativeArgument $_ }) -join ' '
            Write-Host "Round $round/$Rounds, scene $scene, $frames frames"
            $timer = [Diagnostics.Stopwatch]::StartNew()
            $child = Start-Process -FilePath $exePath -ArgumentList $argumentLine -WorkingDirectory $repoRoot -WindowStyle Hidden -PassThru -RedirectStandardOutput $stdoutPath -RedirectStandardError $stderrPath
            # Retain the process handle so ExitCode remains available after exit.
            $null = $child.Handle
            $timedOut = $false
            [long]$sampledWorkingPeak = 0
            [long]$osWorkingPeak = 0
            [long]$privatePeak = 0
            [long]$lastWorking = 0
            [long]$lastPrivate = 0
            $memorySamples = 0
            $focusChecks = 0
            $foregroundChecks = 0
            $focusAcquired = $false
            $focusInterrupted = $false
            $window = [IntPtr]::Zero
            $nextFocusCheckMs = 0
            while (-not $child.HasExited) {
                if ($timer.Elapsed.TotalSeconds -gt $TimeoutSeconds) {
                    $timedOut = $true
                    # Kill only the exact process started by this run.
                    $child.Kill()
                    if (-not $child.WaitForExit(5000)) { throw "Benchmark child $($child.Id) did not exit after timeout." }
                    break
                }
                $child.Refresh()
                if ($child.HasExited) { break }
                $lastWorking = $child.WorkingSet64
                $lastPrivate = $child.PrivateMemorySize64
                $sampledWorkingPeak = [math]::Max($sampledWorkingPeak, $lastWorking)
                $osWorkingPeak = [math]::Max($osWorkingPeak, $child.PeakWorkingSet64)
                $privatePeak = [math]::Max($privatePeak, $lastPrivate)
                $memorySamples++
                if ($timer.ElapsedMilliseconds -ge $nextFocusCheckMs) {
                    $nextFocusCheckMs = $timer.ElapsedMilliseconds + 500
                    if ($window -eq [IntPtr]::Zero) { $window = [PbdBenchmark.Windows]::FindExplorerWindow($child.Id) }
                    if ($window -ne [IntPtr]::Zero) {
                        $focusChecks++
                        $isForeground = [PbdBenchmark.Windows]::GetForegroundWindow() -eq $window
                        if ($isForeground) {
                            $focusAcquired = $true
                            $foregroundChecks++
                        } else {
                            if ($focusAcquired) { $focusInterrupted = $true }
                            $null = [PbdBenchmark.Windows]::SetForegroundWindow($window)
                            if ([PbdBenchmark.Windows]::GetForegroundWindow() -eq $window) {
                                $focusAcquired = $true
                                $foregroundChecks++
                            }
                        }
                    }
                }
                Start-Sleep -Milliseconds 100
            }
            $child.WaitForExit()
            $exitCode = $child.ExitCode
            $timer.Stop()
            $child.Dispose()
            $child = $null
            $stdout = [IO.File]::ReadAllText($stdoutPath)
            $stderr = [IO.File]::ReadAllText($stderrPath)
            $combined = $stdout + "`n" + $stderr
            $timing = [regex]::Match($combined, 'FRAME_WALL_MS p50=([\d.]+) p95=([\d.]+) p99=([\d.]+) samples=(\d+)')
            $errors = [regex]::Matches($combined, '(?im)^.*(?:\bERROR\b|panicked at|Validation Error|CAPTURE_FAILED).*$')
            $tourComplete = -not $isTour -or $combined -match 'RENDERED_TOUR completed=true\b'
            $captureSaved = (Test-Path -LiteralPath $capturePath -PathType Leaf) -and $combined.Contains('CAPTURE_SAVED ')
            $p50 = $null
            $p95 = $null
            $p99 = $null
            $sampleCount = 0
            if ($timing.Success) {
                $p50 = ConvertTo-InvariantDouble $timing.Groups[1].Value
                $p95 = ConvertTo-InvariantDouble $timing.Groups[2].Value
                $p99 = ConvertTo-InvariantDouble $timing.Groups[3].Value
                $sampleCount = [int]$timing.Groups[4].Value
            }
            $valid = $exitCode -eq 0 -and -not $timedOut -and $timing.Success -and $errors.Count -eq 0 -and $tourComplete -and $captureSaved -and $focusAcquired -and -not $focusInterrupted
            $result = [pscustomobject][ordered]@{
                scene = $scene
                round = $round
                frames_requested = $frames
                p50_ms = $p50
                p95_ms = $p95
                p99_ms = $p99
                sample_count = $sampleCount
                p95_within_target = $timing.Success -and $p95 -le $TargetFrameMs
                sampled_peak_working_set_mib = [math]::Round($sampledWorkingPeak / 1MB, 2)
                os_peak_working_set_mib = [math]::Round($osWorkingPeak / 1MB, 2)
                sampled_peak_private_mib = [math]::Round($privatePeak / 1MB, 2)
                last_working_set_mib = [math]::Round($lastWorking / 1MB, 2)
                last_private_mib = [math]::Round($lastPrivate / 1MB, 2)
                memory_samples = $memorySamples
                elapsed_seconds = [math]::Round($timer.Elapsed.TotalSeconds, 3)
                exit_code = $exitCode
                timed_out = $timedOut
                error_count = $errors.Count
                tour_completed = if ($isTour) { $tourComplete } else { $null }
                capture_saved = $captureSaved
                focus_acquired = $focusAcquired
                focus_interrupted = $focusInterrupted
                focus_checks = $focusChecks
                foreground_checks = $foregroundChecks
                valid_run = $valid
                capture = $capturePath
                stdout = $stdoutPath
                stderr = $stderrPath
            }
            $results.Add($result)
            Save-Results
            Write-Host "  p50/p95/p99: $p50 / $p95 / $p99 ms; peak working set: $($result.os_peak_working_set_mib) MiB; valid: $valid"
        }
    }
} finally {
    if ($null -ne $child) {
        if (-not $child.HasExited) {
            $child.Kill()
            $null = $child.WaitForExit(5000)
        }
        $child.Dispose()
    }
    Save-Results
    if (-not $KeepFinalFocus -and [PbdBenchmark.Windows]::IsWindow($initialForeground)) {
        [uint32]$currentOwner = 0
        $null = [PbdBenchmark.Windows]::GetWindowThreadProcessId($initialForeground, [ref]$currentOwner)
        if ($currentOwner -eq $initialForegroundPid) { $null = [PbdBenchmark.Windows]::SetForegroundWindow($initialForeground) }
    }
}

$results | Format-Table scene, round, p50_ms, p95_ms, p99_ms, os_peak_working_set_mib, valid_run -AutoSize
Write-Host "JSON: $resultsPath"
Write-Host "CSV:  $csvPath"
$invalidRuns = @($results | Where-Object { -not $_.valid_run })
if ($invalidRuns.Count -gt 0) { throw "$($invalidRuns.Count) run(s) failed validation; inspect the saved logs and focus checks." }
$overBudget = @($results | Where-Object { -not $_.p95_within_target })
if ($overBudget.Count -gt 0) { Write-Warning "$($overBudget.Count) valid run(s) exceeded the $TargetFrameMs ms p95 target. This is a measured result, not a harness failure." }
