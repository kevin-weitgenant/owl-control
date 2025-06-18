use color_eyre::{
    Result,
    eyre::{Context, bail},
};

use windows::{
    Win32::{
        Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, WPARAM},
        System::LibraryLoader::GetModuleHandleA,
        UI::{
            Input::{
                self, GetRawInputData, HRAWINPUT,
                KeyboardAndMouse::{VK_LBUTTON, VK_MBUTTON, VK_RBUTTON, VK_XBUTTON1, VK_XBUTTON2},
                RAWINPUT, RAWINPUTDEVICE, RAWINPUTHEADER, RID_INPUT, RIDEV_INPUTSINK,
                RegisterRawInputDevices,
            },
            WindowsAndMessaging::{
                CREATESTRUCTA, CreateWindowExA, DefWindowProcA, DestroyWindow, DispatchMessageA,
                GetMessageA, GetWindowLongPtrA, HWND_MESSAGE, MSG, PM_REMOVE, PeekMessageA,
                PostQuitMessage, RI_KEY_BREAK, RI_MOUSE_BUTTON_4_DOWN, RI_MOUSE_BUTTON_4_UP,
                RI_MOUSE_BUTTON_5_DOWN, RI_MOUSE_BUTTON_5_UP, RI_MOUSE_LEFT_BUTTON_DOWN,
                RI_MOUSE_LEFT_BUTTON_UP, RI_MOUSE_MIDDLE_BUTTON_DOWN, RI_MOUSE_MIDDLE_BUTTON_UP,
                RI_MOUSE_RIGHT_BUTTON_DOWN, RI_MOUSE_RIGHT_BUTTON_UP, RI_MOUSE_WHEEL,
                RegisterClassA, SetWindowLongPtrA, TranslateMessage, UnregisterClassA,
                WINDOW_EX_STYLE, WINDOW_LONG_PTR_INDEX, WINDOW_STYLE, WNDCLASSA,
            },
        },
    },
    core::PCSTR,
};

pub struct RawInput<C> {
    hwnd: HWND,
    class_name: PCSTR,
    h_instance: HINSTANCE,
    _marker: std::marker::PhantomData<ProcInfo<C>>,
}

struct ProcInfo<C> {
    event_callback: C,
}

#[derive(Debug, Clone, Copy)]
pub enum Event {
    MouseMove([i32; 2]),
    MousePress {
        key: u16,
        press_state: PressState,
    },
    /// Negative values indicate scrolling down, positive values indicate scrolling up.
    MouseScroll {
        scroll_amount: i16,
    },
    KeyPress {
        key: u16,
        press_state: PressState,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PressState {
    Pressed,
    Released,
}

impl<C> RawInput<C>
where
    C: FnMut(Event),
{
    pub fn initialize(event_callback: C) -> Result<Self> {
        unsafe {
            let class_name = PCSTR(c"RawInputWindowClass".to_bytes_with_nul().as_ptr());
            let h_instance: HINSTANCE = GetModuleHandleA(None)?.into();

            let wc = WNDCLASSA {
                lpfnWndProc: Some(Self::window_proc),
                cbWndExtra: size_of::<*const ProcInfo<C>>()
                    .try_into()
                    .expect("size of &Self should fit in u32"),
                hInstance: h_instance,
                lpszClassName: class_name,
                ..Default::default()
            };

            if RegisterClassA(&wc) == 0 {
                use windows::Win32::Foundation::GetLastError;
                let error = GetLastError();
                bail!("failed to register window class: {error:?}");
            }

            let proc_info = Box::new(ProcInfo { event_callback });
            let proc_info_ptr = Box::into_raw(proc_info);

            tracing::debug!(proc_info_ptr=?proc_info_ptr);

            let hwnd = CreateWindowExA(
                WINDOW_EX_STYLE(0),
                class_name,
                PCSTR::null(),
                WINDOW_STYLE(0),
                0,
                0,
                0,
                0,
                Some(HWND_MESSAGE),
                None,
                Some(h_instance),
                Some(proc_info_ptr as *mut _),
            )
            .wrap_err("failed to create window")?;

            tracing::debug!("RawInput window created: {hwnd:?}");

            let raw_input_devices = [
                0x02, // Mouse
                0x06, // Keyboard
            ]
            .map(|usage| RAWINPUTDEVICE {
                usUsagePage: 0x01, // Generic Desktop Controls
                usUsage: usage,
                dwFlags: RIDEV_INPUTSINK, // Receive input even when not in foreground
                hwndTarget: hwnd,
            });

            RegisterRawInputDevices(
                &raw_input_devices,
                size_of::<RAWINPUTDEVICE>()
                    .try_into()
                    .expect("size of RAWINPUTDEVICE should fit in u32"),
            )
            .wrap_err("failed to register raw input devices")?;

            Ok(Self {
                hwnd,
                class_name,
                h_instance,
                _marker: std::marker::PhantomData,
            })
        }
    }

    pub fn run_queue(&self) -> Result<()> {
        unsafe {
            let mut msg = MSG::default();
            while GetMessageA(&mut msg, None, 0, 0).as_bool() {
                let _ = TranslateMessage(&msg);
                DispatchMessageA(&msg);
            }
            Ok(())
        }
    }

    pub fn poll_queue(&self) -> Result<()> {
        unsafe {
            let mut msg = MSG::default();
            while PeekMessageA(&mut msg, None, 0, 0, PM_REMOVE).as_bool() {
                let _ = TranslateMessage(&msg);
                DispatchMessageA(&msg);
            }
            Ok(())
        }
    }

    #[tracing::instrument(skip_all, fields(hwnd = ?hwnd))]
    unsafe extern "system" fn window_proc(
        hwnd: HWND,
        msg: u32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT {
        unsafe {
            use windows::Win32::UI::WindowsAndMessaging;
            match msg {
                WindowsAndMessaging::WM_CREATE => {
                    let create_struct: *mut CREATESTRUCTA =
                        std::ptr::with_exposed_provenance_mut(lparam.0 as usize);
                    let proc_info: *mut ProcInfo<C> = (*create_struct).lpCreateParams as *mut _;
                    tracing::debug!(
                        msg="WM_CREATE", create_struct=?*create_struct
                    );
                    SetWindowLongPtrA(
                        hwnd,
                        WINDOW_LONG_PTR_INDEX(0),
                        proc_info.expose_provenance() as isize,
                    );
                    LRESULT(0)
                }
                WindowsAndMessaging::WM_DESTROY => {
                    let proc_info_ptr: *mut ProcInfo<C> = std::ptr::with_exposed_provenance_mut(
                        GetWindowLongPtrA(hwnd, WINDOW_LONG_PTR_INDEX(0)) as usize,
                    );
                    let _ = Box::from_raw(proc_info_ptr);
                    tracing::debug!(msg = "WM_DESTROY");
                    PostQuitMessage(0);
                    LRESULT(0)
                }
                WindowsAndMessaging::WM_INPUT => {
                    let hrawinput =
                        HRAWINPUT(std::ptr::with_exposed_provenance_mut(lparam.0 as usize));
                    let mut rawinput = RAWINPUT::default();
                    let mut pcbsize = size_of_val(&rawinput) as u32;
                    let result = GetRawInputData(
                        hrawinput,
                        RID_INPUT,
                        Some(&mut rawinput as *mut _ as *mut _),
                        &mut pcbsize,
                        size_of::<RAWINPUTHEADER>()
                            .try_into()
                            .expect("size of HRAWINPUT should fit in u32"),
                    );
                    if result == u32::MAX {
                        return LRESULT(0);
                    }

                    let proc_info_ptr =
                        GetWindowLongPtrA(hwnd, WINDOW_LONG_PTR_INDEX(0)) as *mut ProcInfo<C>;
                    let proc_info = &mut *proc_info_ptr;
                    let callback = &mut proc_info.event_callback;
                    let mut callback = |event| {
                        tracing::debug!(msg = "WM_INPUT", event=?event);
                        callback(event);
                    };

                    match Input::RID_DEVICE_INFO_TYPE(rawinput.header.dwType) {
                        Input::RIM_TYPEMOUSE => {
                            let mouse = rawinput.data.mouse;
                            if mouse.lLastX != 0 || mouse.lLastY != 0 {
                                callback(Event::MouseMove([mouse.lLastX, mouse.lLastY]));
                            }

                            let us_button_flags =
                                u32::from(mouse.Anonymous.Anonymous.usButtonFlags);

                            if us_button_flags & RI_MOUSE_LEFT_BUTTON_DOWN != 0 {
                                callback(Event::MousePress {
                                    key: VK_LBUTTON.0,
                                    press_state: PressState::Pressed,
                                });
                            } else if us_button_flags & RI_MOUSE_LEFT_BUTTON_UP != 0 {
                                callback(Event::MousePress {
                                    key: VK_LBUTTON.0,
                                    press_state: PressState::Released,
                                });
                            } else if us_button_flags & RI_MOUSE_RIGHT_BUTTON_DOWN != 0 {
                                callback(Event::MousePress {
                                    key: VK_RBUTTON.0,
                                    press_state: PressState::Pressed,
                                });
                            } else if us_button_flags & RI_MOUSE_RIGHT_BUTTON_UP != 0 {
                                callback(Event::MousePress {
                                    key: VK_RBUTTON.0,
                                    press_state: PressState::Released,
                                });
                            } else if us_button_flags & RI_MOUSE_MIDDLE_BUTTON_DOWN != 0 {
                                callback(Event::MousePress {
                                    key: VK_MBUTTON.0,
                                    press_state: PressState::Pressed,
                                });
                            } else if us_button_flags & RI_MOUSE_MIDDLE_BUTTON_UP != 0 {
                                callback(Event::MousePress {
                                    key: VK_MBUTTON.0,
                                    press_state: PressState::Released,
                                });
                            } else if us_button_flags & RI_MOUSE_BUTTON_4_DOWN != 0 {
                                callback(Event::MousePress {
                                    key: VK_XBUTTON1.0,
                                    press_state: PressState::Pressed,
                                });
                            } else if us_button_flags & RI_MOUSE_BUTTON_4_UP != 0 {
                                callback(Event::MousePress {
                                    key: VK_XBUTTON1.0,
                                    press_state: PressState::Released,
                                });
                            } else if us_button_flags & RI_MOUSE_BUTTON_5_DOWN != 0 {
                                callback(Event::MousePress {
                                    key: VK_XBUTTON2.0,
                                    press_state: PressState::Pressed,
                                });
                            } else if us_button_flags & RI_MOUSE_BUTTON_5_UP != 0 {
                                callback(Event::MousePress {
                                    key: VK_XBUTTON2.0,
                                    press_state: PressState::Released,
                                });
                            }

                            if us_button_flags & RI_MOUSE_WHEEL != 0 {
                                callback(Event::MouseScroll {
                                    scroll_amount: mouse.Anonymous.Anonymous.usButtonData as i16,
                                });
                            }
                        }
                        Input::RIM_TYPEKEYBOARD => {
                            let keyboard = rawinput.data.keyboard;
                            let key = keyboard.VKey;
                            let flags = u32::from(keyboard.Flags);
                            let press_state = if flags & RI_KEY_BREAK != 0 {
                                PressState::Released
                            } else {
                                PressState::Pressed
                            };
                            callback(Event::KeyPress { key, press_state });
                        }
                        _ => {}
                    }

                    LRESULT(0)
                }
                _ => DefWindowProcA(hwnd, msg, wparam, lparam),
            }
        }
    }
}

impl<C> Drop for RawInput<C> {
    fn drop(&mut self) {
        unsafe {
            DestroyWindow(self.hwnd).expect("failed to destroy window");
            UnregisterClassA(self.class_name, Some(self.h_instance))
                .expect("failed to unregister class");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore = "run manually"]
    fn print_keypresses() -> Result<()> {
        color_eyre::install()?;
        tracing_subscriber::fmt::init();

        let raw_input = RawInput::initialize(|event| println!("{event:?}"))
            .expect("Failed to initialize raw input");
        raw_input
            .run_queue()
            .wrap_err("failed to run message queue")
    }
}
