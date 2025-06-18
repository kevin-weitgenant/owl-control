mod find_game;
mod hardware_id;
mod input_recorder;
mod recording;

use std::{path::PathBuf, time::Duration};

use clap::Parser;
use color_eyre::{
    Result,
    eyre::{Context as _, OptionExt},
};

use raw_input::RawInput;
use tokio::sync::{mpsc, oneshot};
#[cfg(feature = "real-video")]
use video_audio_recorder::gstreamer;

use crate::{
    find_game::get_foregrounded_game,
    recording::{InputParameters, MetadataParameters, Recording, WindowParameters},
};

#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    #[arg(long)]
    recording_location: PathBuf,

    #[arg(short, long)]
    games: Vec<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    tracing_subscriber::fmt::init();
    #[cfg(feature = "real-video")]
    gstreamer::init()?;

    let Args {
        recording_location,
        games,
    } = Args::parse();

    std::fs::create_dir_all(&recording_location)
        .wrap_err("Failed to create recording directory")?;

    let (pid, hwnd) = get_foregrounded_game(&games)
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

    let mut recording = Recording::start(
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

    let local_set = tokio::task::LocalSet::new();

    let mut input_rx = listen_for_raw_inputs(&local_set);

    let mut stop_rx = wait_for_ctrl_c();

    loop {
        tokio::select! {
            r = &mut stop_rx => {
                r.expect("signal handler was closed early");
                break;
            },
            e = input_rx.recv() => {
                recording.seen_input(e.expect("raw input reader was closed early")).await?;
            },
        }
    }

    tracing::info!("Stopping recording...");

    recording.stop().await?;

    Ok(())
}

fn listen_for_raw_inputs(local_set: &tokio::task::LocalSet) -> mpsc::Receiver<raw_input::Event> {
    let (input_tx, input_rx) = mpsc::channel(1);

    local_set.spawn_local(async move {
        let raw_input = RawInput::initialize(|e| input_tx.blocking_send(e).unwrap())
            .expect("raw input failed to initialize");
        loop {
            raw_input
                .poll_queue()
                .expect("failed to run windows message queue");
            tokio::time::sleep(Duration::from_millis(1000 / 60 / 20)).await;
        }
    });
    input_rx
}

fn wait_for_ctrl_c() -> oneshot::Receiver<()> {
    let (ctrl_c_tx, ctrl_c_rx) = oneshot::channel();

    tokio::spawn(async move {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to listen for Ctrl+C signal");
        let _ = ctrl_c_tx.send(());
    });
    ctrl_c_rx
}
