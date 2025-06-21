New-Item -ItemType Directory -Force -Path build-resources/downloads | Out-Null
Set-Location build-resources/downloads

$downloadManifest = @{
    "vc_redist.x64.exe"                   = "https://aka.ms/vs/17/release/vc_redist.x64.exe"
    "gstreamer-1.0-msvc-x86_64.msi"       = "https://gstreamer.freedesktop.org/data/pkg/windows/1.26.2/msvc/gstreamer-1.0-msvc-x86_64-1.26.2.msi"
    "gstreamer-1.0-devel-msvc-x86_64.msi" = "https://gstreamer.freedesktop.org/data/pkg/windows/1.26.2/msvc/gstreamer-1.0-devel-msvc-x86_64-1.26.2.msi"
    "uv.zip"                              = "https://github.com/astral-sh/uv/releases/download/0.7.12/uv-x86_64-pc-windows-msvc.zip"
}

$downloads = [System.Collections.ArrayList]::new()

$downloadManifest.GetEnumerator() | ForEach-Object {
    $fileName = $_.Key
    $url = $_.Value

    Write-Output "Downloading $fileName from $url"

    $job = Start-Job { 
        $ProgressPreference = 'SilentlyContinue'
        Invoke-WebRequest -OutFile "$using:pwd\$using:fileName" -Uri $using:url
    }
    $downloads.Add($job) | Out-Null
}

# Wait for all downloads to complete
Receive-Job -Wait -Job $downloads

Expand-Archive -Force -LiteralPath uv.zip -DestinationPath uv
