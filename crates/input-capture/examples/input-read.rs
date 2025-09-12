use color_eyre::{Result, eyre::WrapErr as _};
use input_capture::RawInput;

pub fn main() -> Result<()> {
    color_eyre::install()?;
    tracing_subscriber::fmt::init();

    let _raw_input = RawInput::initialize().expect("Failed to initialize raw input");
    RawInput::run_queue(|event| {
        tracing::info!(?event, "Received raw input event");
    })
    .wrap_err("failed to run message queue")
}
