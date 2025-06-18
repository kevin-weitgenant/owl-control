use std::path::PathBuf;

use color_eyre::{
    Result,
    eyre::{Context as _, OptionExt as _},
};
use game_process::Pid;

use crate::{
    find_game::get_foregrounded_game,
    recording::{InputParameters, MetadataParameters, Recording, WindowParameters},
};

pub(crate) struct Recorder<D> {
    recording_dir: D,
    games: Vec<String>,
    recording: Option<InProgressRecording>,
}

struct InProgressRecording {
    recording: Recording,
    pid: Pid,
}

impl<D> Recorder<D>
where
    D: FnMut() -> PathBuf,
{
    pub(crate) fn new(recording_dir: D, games: Vec<String>) -> Self {
        Self {
            recording_dir,
            games,
            recording: None,
        }
    }

    pub(crate) fn is_recording(&self) -> bool {
        self.recording.is_some()
    }

    pub(crate) fn pid(&self) -> Option<Pid> {
        self.recording.as_ref().map(|r| r.pid)
    }

    pub(crate) fn elapsed(&self) -> Option<std::time::Duration> {
        self.recording
            .as_ref()
            .map(|r| r.recording.start_instant().elapsed())
    }

    pub(crate) async fn start(&mut self) -> Result<()> {
        if self.recording.is_some() {
            return Ok(());
        }

        let recording_location = (self.recording_dir)();

        std::fs::create_dir_all(&recording_location)
            .wrap_err("Failed to create recording directory")?;

        let (pid, hwnd) = get_foregrounded_game(&self.games)
            .wrap_err("failed to get foregrounded game")?
            .ok_or_eyre(
                "No game window found. Make sure the game is running and in fullscreen mode.",
            )?;

        tracing::info!(
            pid=?pid,
            hwnd=?hwnd,
            recording_location=%recording_location.display(),
            "Starting recording"
        );

        let recording = Recording::start(
            MetadataParameters {
                path: recording_location.join("metadata.json"),
            },
            WindowParameters {
                path: recording_location.join("recording.mp4"),
                pid,
                hwnd,
            },
            InputParameters {
                path: recording_location.join("inputs.csv"),
            },
        )
        .await?;

        self.recording = Some(InProgressRecording { recording, pid });

        Ok(())
    }

    pub(crate) async fn seen_input(&mut self, e: raw_input::Event) -> Result<()> {
        let Some(InProgressRecording { recording, .. }) = self.recording.as_mut() else {
            return Ok(());
        };
        recording.seen_input(e).await?;
        Ok(())
    }

    pub(crate) async fn stop(&mut self) -> Result<()> {
        let Some(InProgressRecording { recording, .. }) = self.recording.take() else {
            return Ok(());
        };
        recording.stop().await?;
        Ok(())
    }
}
