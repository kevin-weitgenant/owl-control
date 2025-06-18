use color_eyre::Result;
use game_process::hardware_id;

pub(crate) fn get() -> Result<String> {
    hardware_id()
}
