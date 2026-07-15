$ErrorActionPreference = "Stop"
Set-StrictMode -Version Latest

$MaxFileBytes = 536870912
$RuntimeDlls = @(
  "chrome_elf.dll",
  "d3dcompiler_47.dll",
  "dxcompiler.dll",
  "dxil.dll",
  "libEGL.dll",
  "libGLESv2.dll",
  "libcef.dll",
  "vk_swiftshader.dll",
  "vulkan-1.dll"
)

function Assert-RegularFile([string] $Path) {
  $Item = Get-Item -LiteralPath $Path -Force
  if (-not $Item.PSIsContainer -and
      -not ($Item.Attributes -band [IO.FileAttributes]::ReparsePoint) -and
      $Item.Length -gt 0 -and $Item.Length -le $MaxFileBytes) {
    return
  }
  throw "CEF test runtime validation failed"
}

$TauriDir = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot ".."))
$CefRoot = Join-Path $TauriDir ".cef-verified\current"
$TestBinaryDir = Join-Path $TauriDir "target\debug\deps"
if (-not (Test-Path -LiteralPath $CefRoot -PathType Container) -or
    -not (Test-Path -LiteralPath $TestBinaryDir -PathType Container)) {
  throw "CEF test runtime validation failed"
}

foreach ($Name in $RuntimeDlls) {
  $Source = Join-Path $CefRoot $Name
  Assert-RegularFile $Source
  Copy-Item -LiteralPath $Source -Destination (Join-Path $TestBinaryDir $Name) -Force
}

Write-Host "Verified CEF runtime prepared for Windows tests"
