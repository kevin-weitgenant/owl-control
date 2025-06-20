use color_eyre::{Result, eyre::OptionExt};
use game_process::{
    Pid, exe_name_for_pid, foreground_window, is_window_fullscreen,
    windows::Win32::Foundation::HWND,
};

pub(crate) struct Game(String);

impl Game {
    pub(crate) fn new(name: String) -> Self {
        Self(name.to_lowercase())
    }

    pub(crate) fn as_str(&self) -> &str {
        &self.0
    }
}

pub(crate) fn get_foregrounded_game(games: &[Game]) -> Result<Option<(String, Pid, HWND)>> {
    let (hwnd, pid) = foreground_window()?;

    if !is_window_fullscreen(hwnd)? {
        return Ok(None);
    }

    let exe_path = exe_name_for_pid(pid)?;
    let exe_name = exe_path
        .file_name()
        .ok_or_eyre("Failed to get file name from exe path")?
        .to_str()
        .ok_or_eyre("Failed to convert exe name to unicode string")?
        .to_owned();
    if !is_process_game(&exe_name, games) {
        return Ok(None);
    }

    Ok(Some((exe_name, pid, hwnd)))
}

fn is_process_game(name: &str, games: &[Game]) -> bool {
    let name = name.to_lowercase();
    games.iter().any(|game| name.contains(game.as_str()))
}
