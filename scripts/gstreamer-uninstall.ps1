Start-Process msiexec -Wait -ArgumentList '/passive /uninstall build-resources\downloads\gstreamer-1.0-msvc-x86_64.msi'
Start-Process msiexec -Wait -ArgumentList '/passive /uninstall build-resources\downloads\gstreamer-1.0-devel-msvc-x86_64.msi'
