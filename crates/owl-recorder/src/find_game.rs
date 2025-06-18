use color_eyre::Result;
use game_process::{
    Pid, foreground_window, is_window_fullscreen, process_name_for_pid,
    windows::Win32::Foundation::HWND,
};

pub(crate) fn get_foregrounded_game(games: &[String]) -> Result<Option<(Pid, HWND)>> {
    let (hwnd, pid) = foreground_window()?;

    if !is_window_fullscreen(hwnd)? {
        return Ok(None);
    }

    let process_name = process_name_for_pid(pid)?;
    if !is_process_game(process_name.as_bytes(), games) {
        return Ok(None);
    }

    Ok(Some((pid, hwnd)))
}

fn is_process_game(name: &[u8], games: &[String]) -> bool {
    let Ok(exe_file) = str::from_utf8(name)
        .inspect_err(|e| tracing::debug!("Failed to convert process name to string: {e}"))
    else {
        return false;
    };
    games.iter().any(|game| exe_file.contains(game))
}
