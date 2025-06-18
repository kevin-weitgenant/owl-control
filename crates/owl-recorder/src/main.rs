mod hardware_id;
mod input_recorder;
mod recording;

use std::{path::PathBuf, time::Duration};

use clap::Parser;
use color_eyre::{Result, eyre::Context as _};

use raw_input::RawInput;
use tokio::sync::{mpsc, oneshot};
#[cfg(feature = "real-video")]
use video_audio_recorder::gstreamer;

use crate::recording::{InputParameters, MetadataParameters, Recording, WindowParameters};

#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    #[arg(long)]
    recording_location: PathBuf,

    #[arg(long)]
    pid: u32,

    #[arg(long)]
    hwnd: u32,
}

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    tracing_subscriber::fmt::init();
    #[cfg(feature = "real-video")]
    gstreamer::init()?;

    let Args {
        recording_location,
        pid,
        hwnd,
    } = Args::parse();

    std::fs::create_dir_all(&recording_location)
        .wrap_err("Failed to create recording directory")?;

    tracing::info!(
        pid=?pid,
        hwnd=?hwnd,
        recording_location=%recording_location.display(),
        "Started recording"
    );

    let file_in_recording_location = |name| {
        let mut recording_location = recording_location.clone();
        recording_location.set_file_name(name);
        recording_location
    };

    let mut recording = Recording::start(
        MetadataParameters {
            path: file_in_recording_location("metadata.json"),
        },
        WindowParameters {
            path: file_in_recording_location("recording.mp4"),
            pid,
            hwnd,
        },
        InputParameters {
            path: file_in_recording_location("inputs.csv"),
        },
    )
    .await?;

    let (input_tx, mut input_rx) = mpsc::channel(1);

    tokio::task::spawn_local(async move {
        let raw_input = RawInput::initialize(|e| input_tx.blocking_send(e).unwrap())
            .expect("raw input failed to initialize");
        loop {
            raw_input
                .run_message_queue_till_empty()
                .expect("failed to run windows message queue");
            tokio::time::sleep(Duration::from_millis(1000 / 60 / 20)).await;
        }
    });

    let (stop_tx, mut stop_rx) = oneshot::channel();

    tokio::spawn(async move {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to listen for Ctrl+C signal");
        let _ = stop_tx.send(());
    });

    loop {
        tokio::select! {
            r = &mut stop_rx => {
                r.expect("signal handler was closed early");
                break;
            },
            e = (&mut input_rx).recv() => {
                recording.seen_input(e.expect("raw input reader was closed early")).await?;
            },
        }
    }

    tracing::info!("Stopping recording...");

    recording.stop().await?;

    Ok(())
}
