New-Item -ItemType Directory -Force -Path build-resources/downloads | Out-Null

# vc_redist
curl -o build-resources/downloads/vc_redist.x64.exe https://aka.ms/vs/17/release/vc_redist.x64.exe

# gstreamer
curl -o build-resources/downloads/gstreamer-1.0-msvc-x86_64.msi https://gstreamer.freedesktop.org/data/pkg/windows/1.26.2/msvc/gstreamer-1.0-msvc-x86_64-1.26.2.msi
