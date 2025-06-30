# Onboarding

WIP

## Building the Rust recorder

In order to build the Rust recorder, gstreamer must be installed and gstreamer's `pkg_config.exe` must be on your path.

```powershell
npm run gstreamer:install

# puts pkg_config and the gstreamer DLL in your path for the current shell session
$env:PATH = "$GSTREAMER_1_0_ROOT_MSVC_X86_64\\bin;$PATH"
```
