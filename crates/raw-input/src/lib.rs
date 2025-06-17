use color_eyre::{
    Result,
    eyre::{Context, bail},
};

use windows::{
    Win32::{
        Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, WPARAM},
        System::{LibraryLoader::GetModuleHandleA, SystemInformation::GetTickCount64},
        UI::{
            Input::{
                self, GetRawInputData, HRAWINPUT,
                KeyboardAndMouse::{VK_LBUTTON, VK_MBUTTON, VK_RBUTTON, VK_XBUTTON1, VK_XBUTTON2},
                RAWINPUT, RAWINPUTDEVICE, RID_INPUT, RIDEV_INPUTSINK, RegisterRawInputDevices,
            },
            WindowsAndMessaging::{
                CREATESTRUCTA, CreateWindowExA, DefWindowProcA, DestroyWindow, GetWindowLongPtrA,
                HWND_MESSAGE, PostQuitMessage, RI_KEY_BREAK, RI_MOUSE_BUTTON_4_DOWN,
                RI_MOUSE_BUTTON_4_UP, RI_MOUSE_BUTTON_5_DOWN, RI_MOUSE_BUTTON_5_UP,
                RI_MOUSE_LEFT_BUTTON_DOWN, RI_MOUSE_LEFT_BUTTON_UP, RI_MOUSE_MIDDLE_BUTTON_DOWN,
                RI_MOUSE_MIDDLE_BUTTON_UP, RI_MOUSE_RIGHT_BUTTON_DOWN, RI_MOUSE_RIGHT_BUTTON_UP,
                RI_MOUSE_WHEEL, RegisterClassExA, SetWindowLongPtrA, UnregisterClassA,
                WINDOW_EX_STYLE, WINDOW_LONG_PTR_INDEX, WINDOW_STYLE, WNDCLASSEXA,
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
pub struct Event {
    pub event_type: EventType,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Copy)]
pub enum EventType {
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

#[derive(Debug, Clone, Copy)]
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

            let wc = WNDCLASSEXA {
                lpfnWndProc: Some(Self::window_proc),
                cbWndExtra: size_of::<*const ProcInfo<C>>()
                    .try_into()
                    .expect("size of &Self should fit in u32"),
                hInstance: h_instance,
                lpszClassName: class_name,
                ..Default::default()
            };

            if RegisterClassExA(&wc) == 0 {
                bail!("failed to register window class");
            }

            let proc_info = Box::new(ProcInfo { event_callback });

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
                Some(Box::into_raw(proc_info) as *mut _),
            )
            .wrap_err("failed to create window")?;

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
                    let create_struct = lparam.0 as *mut CREATESTRUCTA;
                    let lp_create_param = (*create_struct).lpCreateParams;
                    SetWindowLongPtrA(hwnd, WINDOW_LONG_PTR_INDEX(0), lp_create_param as isize);
                    LRESULT(0)
                }
                WindowsAndMessaging::WM_DESTROY => {
                    let proc_info_ptr =
                        GetWindowLongPtrA(hwnd, WINDOW_LONG_PTR_INDEX(0)) as *mut ProcInfo<C>;
                    let _ = Box::from_raw(proc_info_ptr);
                    PostQuitMessage(0);
                    LRESULT(0)
                }
                WindowsAndMessaging::WM_INPUT => {
                    let hrawinput = *(lparam.0 as *mut HRAWINPUT);
                    let mut rawinput = RAWINPUT::default();
                    let mut pcbsize = size_of_val(&rawinput) as u32;
                    let result = GetRawInputData(
                        hrawinput,
                        RID_INPUT,
                        Some(&mut rawinput as *mut _ as *mut _),
                        &mut pcbsize,
                        size_of_val(&hrawinput)
                            .try_into()
                            .expect("size of HRAWINPUT should fit in u32"),
                    );
                    if result == u32::MAX {
                        return LRESULT(0);
                    }

                    let timestamp = GetTickCount64();

                    let proc_info_ptr =
                        GetWindowLongPtrA(hwnd, WINDOW_LONG_PTR_INDEX(0)) as *mut ProcInfo<C>;
                    let proc_info = &mut *proc_info_ptr;
                    let mut callback = |event_type| {
                        (proc_info.event_callback)(Event {
                            event_type,
                            timestamp,
                        })
                    };

                    match Input::RID_DEVICE_INFO_TYPE(rawinput.header.dwType) {
                        Input::RIM_TYPEMOUSE => {
                            let mouse = rawinput.data.mouse;
                            if mouse.lLastX != 0 || mouse.lLastY != 0 {
                                callback(EventType::MouseMove([mouse.lLastX, mouse.lLastY]));
                            }

                            let us_button_flags =
                                u32::from(mouse.Anonymous.Anonymous.usButtonFlags);

                            if us_button_flags & RI_MOUSE_LEFT_BUTTON_DOWN != 0 {
                                callback(EventType::MousePress {
                                    key: VK_LBUTTON.0,
                                    press_state: PressState::Pressed,
                                });
                            } else if us_button_flags & RI_MOUSE_LEFT_BUTTON_UP != 0 {
                                callback(EventType::MousePress {
                                    key: VK_LBUTTON.0,
                                    press_state: PressState::Released,
                                });
                            } else if us_button_flags & RI_MOUSE_RIGHT_BUTTON_DOWN != 0 {
                                callback(EventType::MousePress {
                                    key: VK_RBUTTON.0,
                                    press_state: PressState::Pressed,
                                });
                            } else if us_button_flags & RI_MOUSE_RIGHT_BUTTON_UP != 0 {
                                callback(EventType::MousePress {
                                    key: VK_RBUTTON.0,
                                    press_state: PressState::Released,
                                });
                            } else if us_button_flags & RI_MOUSE_MIDDLE_BUTTON_DOWN != 0 {
                                callback(EventType::MousePress {
                                    key: VK_MBUTTON.0,
                                    press_state: PressState::Pressed,
                                });
                            } else if us_button_flags & RI_MOUSE_MIDDLE_BUTTON_UP != 0 {
                                callback(EventType::MousePress {
                                    key: VK_MBUTTON.0,
                                    press_state: PressState::Released,
                                });
                            } else if us_button_flags & RI_MOUSE_BUTTON_4_DOWN != 0 {
                                callback(EventType::MousePress {
                                    key: VK_XBUTTON1.0,
                                    press_state: PressState::Pressed,
                                });
                            } else if us_button_flags & RI_MOUSE_BUTTON_4_UP != 0 {
                                callback(EventType::MousePress {
                                    key: VK_XBUTTON1.0,
                                    press_state: PressState::Released,
                                });
                            } else if us_button_flags & RI_MOUSE_BUTTON_5_DOWN != 0 {
                                callback(EventType::MousePress {
                                    key: VK_XBUTTON2.0,
                                    press_state: PressState::Pressed,
                                });
                            } else if us_button_flags & RI_MOUSE_BUTTON_5_UP != 0 {
                                callback(EventType::MousePress {
                                    key: VK_XBUTTON2.0,
                                    press_state: PressState::Released,
                                });
                            }

                            if us_button_flags & RI_MOUSE_WHEEL != 0 {
                                callback(EventType::MouseScroll {
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
                            callback(EventType::KeyPress { key, press_state });
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
