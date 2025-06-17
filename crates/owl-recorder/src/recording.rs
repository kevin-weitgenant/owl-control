use std::{
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use color_eyre::Result;
use serde::Serialize;

#[cfg(feature = "real-video")]
use video_audio_recorder::WindowRecorder;

use crate::{hardware_id, input_recorder::InputRecorder};

pub(crate) struct Recording {
    #[cfg(feature = "real-video")]
    window_recorder: WindowRecorder,
    #[cfg(feature = "real-video")]
    window_recorder_listener: AbortOnDropHandle<()>,
    input_recorder: InputRecorder,

    metadata_path: PathBuf,
    start_time: SystemTime,
}

pub(crate) struct MetadataParameters {
    pub(crate) path: PathBuf,
}

pub(crate) struct WindowParameters {
    pub(crate) path: PathBuf,
    pub(crate) pid: u32,
    pub(crate) hwnd: u32,
}

pub(crate) struct InputParameters {
    pub(crate) path: PathBuf,
}

impl Recording {
    pub(crate) async fn start(
        MetadataParameters {
            path: metadata_path,
        }: MetadataParameters,
        #[cfg_attr(not(feature = "real-video"), expect(unused_variables))] WindowParameters {
            path: video_path,
            pid,
            hwnd,
        }: WindowParameters,
        InputParameters { path: csv_path }: InputParameters,
    ) -> Result<Self> {
        let start_time = SystemTime::now();

        #[cfg(feature = "real-video")]
        let window_recorder = WindowRecorder::start_recording(&video_path, pid, hwnd)?;
        #[cfg(feature = "real-video")]
        let window_recorder_listener =
            AbortOnDropHandle(tokio::spawn(window_recorder.listen_to_messages()));

        let input_recorder = InputRecorder::start(&csv_path).await?;

        Ok(Self {
            #[cfg(feature = "real-video")]
            window_recorder,
            #[cfg(feature = "real-video")]
            window_recorder_listener,

            input_recorder,

            metadata_path,
            start_time,
        })
    }

    pub(crate) async fn seen_input(&mut self, e: raw_input::Event) -> Result<()> {
        self.input_recorder.seen_input(e).await
    }

    pub(crate) async fn stop(self) -> Result<()> {
        #[cfg(feature = "real-video")]
        self.window_recorder.stop_recording();
        #[cfg(feature = "real-video")]
        self.window_recorder_listener.await?;

        self.input_recorder.stop().await?;

        Self::write_metadata(self.start_time, &self.metadata_path).await?;
        Ok(())
    }

    async fn write_metadata(start_time: SystemTime, path: &Path) -> Result<()> {
        let metadata = Self::final_metadata(start_time).await?;
        let metadata = serde_json::to_string_pretty(&metadata)?;
        tokio::fs::write(path, &metadata).await?;
        Ok(())
    }

    async fn final_metadata(start_time: SystemTime) -> Result<Metadata> {
        let end_time = SystemTime::now();
        let duration = end_time
            .duration_since(start_time)
            .expect("start time was recorded earlier")
            .as_secs();

        let start_timestamp = start_time.duration_since(UNIX_EPOCH).unwrap().as_secs();
        let end_timestamp = start_time.duration_since(UNIX_EPOCH).unwrap().as_secs();

        let hardware_id = hardware_id::get().await?;

        Ok(Metadata {
            session_id: uuid::Uuid::new_v4().to_string(),
            hardware_id,
            start_timestamp,
            end_timestamp,
            duration,
        })
    }
}

#[derive(Serialize)]
struct Metadata {
    session_id: String,
    hardware_id: String,
    start_timestamp: u64,
    end_timestamp: u64,
    duration: u64,
}
