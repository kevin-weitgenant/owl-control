use std::path::PathBuf;

use clap::Parser;
use color_eyre::{
    Result,
    eyre::{Context as _, OptionExt as _},
};

use owl_recorder::WindowRecorder;

#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    #[arg(long)]
    video_location: PathBuf,

    #[arg(long)]
    pid: u32,

    #[arg(long)]
    hwnd: u32,
}

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    tracing_subscriber::fmt::init();

    let args = Args::parse();

    std::fs::create_dir_all(
        &args
            .video_location
            .parent()
            .ok_or_eyre("Root isn't a valid file!")?,
    )
    .wrap_err("Failed to create video location directory")?;

    let recorder = WindowRecorder::start_recording(&args.video_location, args.pid, args.hwnd)?;

    let listener_task = tokio::task::spawn(recorder.listen_to_messages());

    tokio::signal::ctrl_c()
        .await
        .wrap_err("Failed to listen for Ctrl+C signal")?;

    tracing::info!("Stopping recording...");

    recorder.stop_recording();

    listener_task.await.unwrap()?;

    Ok(())
}
