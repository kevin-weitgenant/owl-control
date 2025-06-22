Start-Process msiexec -Wait -ArgumentList '/passive /i build-resources\downloads\gstreamer-1.0-msvc-x86_64.msi ADDLOCAL=ALL'
Start-Process msiexec -Wait -ArgumentList '/passive /i build-resources\downloads\gstreamer-1.0-devel-msvc-x86_64.msi'
