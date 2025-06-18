use bstr::{BStr, BString, ByteSlice as _};
use color_eyre::Result;
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

pub(crate) fn get_foregrounded_game(games: &[Game]) -> Result<Option<(BString, Pid, HWND)>> {
    let (hwnd, pid) = foreground_window()?;

    if !is_window_fullscreen(hwnd)? {
        return Ok(None);
    }

    let exe_name = BString::new(exe_name_for_pid(pid)?.into_bytes());
    if !is_process_game(exe_name.as_bstr(), games) {
        return Ok(None);
    }

    Ok(Some((exe_name, pid, hwnd)))
}

fn is_process_game(name: &BStr, games: &[Game]) -> bool {
    games
        .iter()
        .any(|game| name.to_lowercase().contains_str(game.as_str()))
}
