use std::{mem, os::raw::c_void};
use windows::{
    Win32::{
        Devices::HumanInterfaceDevice::*,
        Foundation::{E_FAIL, HWND},
        System::{Com::*, LibraryLoader::GetModuleHandleW},
    },
    core::*,
};

pub struct DirectInput {
    _dinput: IDirectInput8W,
    device: Option<IDirectInputDevice8W>,
    last_state: Option<DIJOYSTATE2>,
}
impl DirectInput {
    pub fn new() -> color_eyre::Result<Self> {
        unsafe {
            // Initialize COM
            CoInitialize(None).ok()?;

            // Create DirectInput
            let mut dinput: Option<IDirectInput8W> = None;
            DirectInput8Create(
                GetModuleHandleW(None)?.into(),
                DIRECTINPUT_VERSION,
                &IDirectInput8W::IID,
                &mut dinput as *mut _ as _,
                None,
            )?;
            let dinput = dinput.ok_or_else(|| Error::from_hresult(E_FAIL))?;

            // Find first gamepad
            struct EnumContext {
                guids: Vec<GUID>,
            }
            unsafe extern "system" fn enum_devices_callback(
                instance: *mut DIDEVICEINSTANCEW,
                context: *mut c_void,
            ) -> BOOL {
                unsafe {
                    let context = &mut *(context as *mut EnumContext);
                    context.guids.push((*instance).guidInstance);
                }
                BOOL(DIENUM_CONTINUE as _)
            }
            let mut enum_context = EnumContext { guids: Vec::new() };
            dinput.EnumDevices(
                DI8DEVCLASS_GAMECTRL,
                Some(enum_devices_callback),
                &mut enum_context as *mut _ as _,
                DIEDFL_ATTACHEDONLY,
            )?;

            dbg!(&enum_context.guids);

            // Create the device if it exists
            let device = if let Some(guid) = enum_context.guids.first() {
                let mut device: Option<IDirectInputDevice8W> = None;
                dinput.CreateDevice(guid, &mut device, None)?;

                let device = device.ok_or_else(|| Error::from_hresult(E_FAIL))?;

                // We use C++ code for this as the function we're calling requires
                // some data that is not exposed by windows-rs
                unsafe extern "C" {
                    unsafe fn set_device_data_format(device: *mut c_void);
                }
                set_device_data_format(device.as_raw());
                device
                    .SetCooperativeLevel(HWND::default(), DISCL_BACKGROUND | DISCL_NONEXCLUSIVE)?;
                device.Acquire()?;

                Some(device)
            } else {
                None
            };

            Ok(Self {
                _dinput: dinput,
                device,
                last_state: None,
            })
        }
    }

    pub fn update(&mut self) {
        let Some(device) = self.device.as_mut() else {
            return;
        };

        unsafe {
            device.Poll().ok();

            let mut state: DIJOYSTATE2 = mem::zeroed();
            if let Err(_) = device.GetDeviceState(
                mem::size_of::<DIJOYSTATE2>() as u32,
                &mut state as *mut _ as _,
            ) {
                tracing::warn!("failed to get device state");
                device.Acquire().ok(); // Try to reacquire
                return;
            }

            let Some(last_state) = self.last_state else {
                self.last_state = Some(state);
                return;
            };

            diff_scalar("lX", last_state.lX, state.lX);
            diff_scalar("lY", last_state.lY, state.lY);
            diff_scalar("lZ", last_state.lZ, state.lZ);
            diff_scalar("lRx", last_state.lRx, state.lRx);
            diff_scalar("lRy", last_state.lRy, state.lRy);
            diff_scalar("lRz", last_state.lRz, state.lRz);
            diff_array_of_scalar("rglSlider", last_state.rglSlider, state.rglSlider);
            diff_array_of_scalar("rgdwPOV", last_state.rgdwPOV, state.rgdwPOV);
            diff_array_of_scalar("rgbButtons", last_state.rgbButtons, state.rgbButtons);
            diff_scalar("lVX", last_state.lVX, state.lVX);
            diff_scalar("lVY", last_state.lVY, state.lVY);
            diff_scalar("lVZ", last_state.lVZ, state.lVZ);
            diff_scalar("lVRx", last_state.lVRx, state.lVRx);
            diff_scalar("lVRy", last_state.lVRy, state.lVRy);
            diff_scalar("lVRz", last_state.lVRz, state.lVRz);
            diff_array_of_scalar("rglVSlider", last_state.rglVSlider, state.rglVSlider);
            diff_scalar("lAX", last_state.lAX, state.lAX);
            diff_scalar("lAY", last_state.lAY, state.lAY);
            diff_scalar("lAZ", last_state.lAZ, state.lAZ);
            diff_scalar("lARx", last_state.lARx, state.lARx);
            diff_scalar("lARy", last_state.lARy, state.lARy);
            diff_scalar("lARz", last_state.lARz, state.lARz);
            diff_array_of_scalar("rglASlider", last_state.rglASlider, state.rglASlider);
            diff_scalar("lFX", last_state.lFX, state.lFX);
            diff_scalar("lFY", last_state.lFY, state.lFY);
            diff_scalar("lFZ", last_state.lFZ, state.lFZ);
            diff_scalar("lFRx", last_state.lFRx, state.lFRx);
            diff_scalar("lFRy", last_state.lFRy, state.lFRy);
            diff_scalar("lFRz", last_state.lFRz, state.lFRz);
            diff_array_of_scalar("rglFSlider", last_state.rglFSlider, state.rglFSlider);

            self.last_state = Some(state);
        }
    }
}

fn diff_scalar<T: std::ops::Sub<Output = T> + PartialOrd + PartialEq + Eq + std::fmt::Display>(
    name: &str,
    a: T,
    b: T,
) {
    if a != b {
        println!("{name}: {a} -> {b}");
    }
}

fn diff_array_of_scalar<
    T: std::ops::Sub<Output = T> + PartialOrd + PartialEq + Eq + std::fmt::Display + Copy,
    const N: usize,
>(
    name: &str,
    a: [T; N],
    b: [T; N],
) {
    for (i, (a, b)) in a.iter().zip(b.iter()).enumerate() {
        diff_scalar(format!("{name}[{i}]").as_str(), *a, *b);
    }
}
