// windows-rs doesn't have the necessary global constants bound,
// so this is a short detour into C++ land to call this function
// with the correct state and then return to Rust land

#include <windows.h>
#include <dinput.h>

extern "C" void set_device_data_format(IDirectInputDevice8W* device) {
    device->SetDataFormat(&c_dfDIJoystick2);
}