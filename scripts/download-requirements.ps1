New-Item -ItemType Directory -Force -Path build-resources/downloads | Out-Null
Set-Location build-resources/downloads

$downloads = [System.Collections.ArrayList]::new()

# vc_redist
$downloads.Add((Start-Job { Invoke-WebRequest -o $using:pwd\vc_redist.x64.exe https://aka.ms/vs/17/release/vc_redist.x64.exe })) 

# gstreamer
$downloads.Add((Start-Job { Invoke-WebRequest -o $using:pwd\gstreamer-1.0-msvc-x86_64.msi https://gstreamer.freedesktop.org/data/pkg/windows/1.26.2/msvc/gstreamer-1.0-msvc-x86_64-1.26.2.msi }))

# uv
$downloads.Add((Start-Job { Invoke-WebRequest -o $using:pwd\uv.zip https://github.com/astral-sh/uv/releases/download/0.7.12/uv-x86_64-pc-windows-msvc.zip }))

# Wait for all downloads to complete
Receive-Job -Wait -Job $downloads

Expand-Archive -Force -LiteralPath uv.zip -DestinationPath uv
