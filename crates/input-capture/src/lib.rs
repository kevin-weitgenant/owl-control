mod raw_input;
use tokio::sync::mpsc;

use raw_input::RawInput;

use color_eyre::Result;

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
impl Event {
    pub fn key_press_keycode(&self) -> Option<u16> {
        match self {
            Event::KeyPress {
                key,
                press_state: PressState::Pressed,
            } => Some(*key),
            _ => None,
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PressState {
    Pressed,
    Released,
}

pub struct InputCapture {
    _capture_thread: std::thread::JoinHandle<()>,
}
impl InputCapture {
    pub fn new() -> Result<(Self, mpsc::Receiver<Event>)> {
        let (input_tx, input_rx) = mpsc::channel(10);

        let _capture_thread = std::thread::spawn(move || {
            let mut raw_input =
                Some(RawInput::initialize().expect("failed to initialize raw input"));
            RawInput::run_queue(move |event| {
                if input_tx.blocking_send(event).is_err() {
                    // Force the raw input to be dropped, which will unregister the window
                    // class and destroy the window, stopping the message queue.
                    raw_input.take();
                }
            })
            .expect("failed to run windows message queue");
        });

        Ok((Self { _capture_thread }, input_rx))
    }
}
