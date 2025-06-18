mod find_game;
mod hardware_id;
mod idle;
mod input_recorder;
mod keycode;
mod recorder;
mod recording;

use std::{
    path::PathBuf,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use clap::Parser;
use color_eyre::{Result, eyre::eyre};

use game_process::does_process_exist;
use raw_input::RawInput;
use tokio::{
    sync::{mpsc, oneshot},
    time::MissedTickBehavior,
};
#[cfg(feature = "real-video")]
use video_audio_recorder::gstreamer;

use crate::{idle::IdlenessTracker, keycode::lookup_keycode, recorder::Recorder};

#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    #[arg(long)]
    recording_location: PathBuf,

    #[arg(short, long)]
    games: Vec<String>,

    #[arg(long, default_value = "F4")]
    start_hotkey: String,

    #[arg(long, default_value = "F5")]
    stop_hotkey: String,
}

const MAX_IDLE_DURATION: Duration = Duration::from_secs(5);
const MAX_RECORDING_DURATION: Duration = Duration::from_secs(60 * 20);

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    tracing_subscriber::fmt::init();
    #[cfg(feature = "real-video")]
    gstreamer::init()?;

    let Args {
        recording_location,
        games,
        start_hotkey,
        stop_hotkey,
    } = Args::parse();

    let start_hotkey = lookup_keycode(&start_hotkey)
        .ok_or_else(|| eyre!("Invalid start hotkey: {start_hotkey}"))?;
    let end_hotkey =
        lookup_keycode(&stop_hotkey).ok_or_else(|| eyre!("Invalid stop hotkey: {stop_hotkey}"))?;

    let mut recorder = Recorder::new(
        || {
            recording_location.join(
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs()
                    .to_string(),
            )
        },
        games,
    );

    let mut input_rx = listen_for_raw_inputs();

    let mut stop_rx = wait_for_ctrl_c();

    let mut idleness_tracker = IdlenessTracker::new(MAX_IDLE_DURATION);

    let mut perform_checks = tokio::time::interval(Duration::from_secs(1));
    perform_checks.set_missed_tick_behavior(MissedTickBehavior::Delay);

    loop {
        tokio::select! {
            r = &mut stop_rx => {
                r.expect("signal handler was closed early");
                break;
            },
            e = input_rx.recv() => {
                let e = e.expect("raw input reader was closed early");
                recorder.seen_input(e).await?;
                if let Some(key) = keycode_from_event(&e) {
                    if key == start_hotkey {
                        tracing::info!("Start hotkey pressed, starting recording");
                        recorder.start().await?;
                    } else if key == end_hotkey {
                        tracing::info!("Stop hotkey pressed, stopping recording");
                        recorder.stop().await?;
                    }
                }
                if idleness_tracker.is_idle() {
                    tracing::info!("Input detected, restarting recording");
                    recorder.start().await?;
                }
                idleness_tracker.update_activity();
            },
            _ = perform_checks.tick(), if recorder.is_recording() => {
                if idleness_tracker.is_idle() {
                    tracing::info!("No input detected for 5 seconds, stopping recording");
                    recorder.stop().await?;
                }

                if let Some(e) = recorder.elapsed() {
                    if e > MAX_RECORDING_DURATION {
                        tracing::info!("Recording duration exceeded {} s, restarting recording", MAX_RECORDING_DURATION.as_secs());
                        recorder.stop().await?;
                        recorder.start().await?;
                        idleness_tracker.update_activity();
                    }
                };

                if let Some(pid) = recorder.pid() {
                    if !does_process_exist(pid)? {
                        tracing::info!(pid=pid.0, "Game process no longer exists, stopping recording");
                        recorder.stop().await?;
                    }
                }
            },
        }
    }

    recorder.stop().await?;

    Ok(())
}

fn keycode_from_event(event: &raw_input::Event) -> Option<u16> {
    if let raw_input::Event::KeyPress { key, .. } = event {
        Some(*key)
    } else {
        None
    }
}

fn listen_for_raw_inputs() -> mpsc::Receiver<raw_input::Event> {
    let (input_tx, input_rx) = mpsc::channel(1);

    std::thread::spawn(move || {
        let mut raw_input = Some(RawInput::initialize().expect("raw input failed to initialize"));

        RawInput::run_queue(|event| {
            if input_tx.blocking_send(event).is_err() {
                tracing::debug!("Input channel closed, stopping raw input listener");
                raw_input.take();
            }
        })
        .expect("failed to run windows message queue");
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
