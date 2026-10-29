# Downloads the DocAligner document-corner model used by "Scan document".
# Source: https://github.com/DocsaidLab/DocAligner (code: Apache-2.0).
# The weights are fetched from the DocAligner project's own Google Drive folder.
# Without the model the app still works and falls back to classic edge detection.
$ErrorActionPreference = 'Stop'
$hedef = Join-Path $PSScriptRoot '..\src-tauri\modeller\docaligner-fastvit_sa24.onnx'
$id = '14vUH77v6yGg7zFctUgcT6BzV5Iisg4Dl'   # fastvit_sa24_h_e_bifpn_256_fp32.onnx
if (Test-Path $hedef) { Write-Host "Model already present: $hedef"; exit 0 }
New-Item -ItemType Directory -Force (Split-Path $hedef) | Out-Null
Write-Host 'Downloading DocAligner fastvit_sa24 (~83 MB)...'
Invoke-WebRequest -Uri "https://drive.usercontent.google.com/download?id=$id&export=download&confirm=t" -OutFile $hedef
$boyut = (Get-Item $hedef).Length
if ($boyut -lt 50MB) { Remove-Item $hedef; throw "Download looks wrong ($boyut bytes). Get the file manually from the DocAligner project." }
Write-Host "Saved $([math]::Round($boyut / 1MB, 1)) MB to $hedef"
