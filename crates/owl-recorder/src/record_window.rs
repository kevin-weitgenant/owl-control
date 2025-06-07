use std::path::Path;

use color_eyre::{
    Result,
    eyre::{Context, ContextCompat as _, OptionExt as _},
};
use gstreamer::{
    Pipeline,
    glib::object::Cast,
    prelude::{ElementExt as _, ElementExtManual as _, GObjectExtManualGst as _, GstBinExt as _},
};

pub struct WindowRecorder {
    element: gstreamer::Element,
}

impl WindowRecorder {
    pub fn start_recording(path: &Path, pid: u32, hwnd: u32) -> Result<Self> {
        gstreamer::init()?;

        let element = WindowRecorder::create_pipeline(path, pid, hwnd)?;
        element
            .set_state(gstreamer::State::Playing)
            .wrap_err("failed to set pipeline state to Playing")?;
        Ok(WindowRecorder { element })
    }

    pub fn listen_to_messages(&self) -> impl Future<Output = Result<()>> + use<> {
        let element = self.element.clone();
        let bus = self.element.bus().unwrap();
        async move {
            for msg in bus.iter_timed(gstreamer::ClockTime::NONE) {
                use gstreamer::MessageView;

                match msg.view() {
                    MessageView::Eos(..) => {
                        tracing::debug!("received eos");
                        break;
                    }
                    MessageView::Error(err) => {
                        tracing::error!(message = "pipeline error", error = ?err.error());
                        break;
                    }
                    _ => (),
                };

                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            }
            element
                .set_state(gstreamer::State::Null)
                .wrap_err("Failed to set pipeline state to Null")?;
            Ok(())
        }
    }

    pub fn stop_recording(self) {
        self.element.send_event(gstreamer::event::Eos::new());
    }

    fn create_pipeline(path: &Path, _pid: u32, hwnd: u32) -> Result<gstreamer::Element> {
        // Loopback is bugged: gstreamer/gstreamer#4259
        // Add the following parameters once it's fixed: "loopback-target-pid={pid} loopback-mode=include-process-tree"
        let video = format!(
            "
            d3d12screencapturesrc window-handle={hwnd} capture-api=wgc
            ! d3d12convert
            ! d3d12download
            ! video/x-raw,width=1920,height=1080
            ! nvh264enc
            ! h264parse
            ! queue ! mux.

            wasapi2src
            ! audioconvert
            ! audioresample
            ! audio/x-raw,format=S16LE,channels=2,rate=48000
            ! lamemp3enc
            ! queue ! mux.

            mp4mux name=mux
            ! filesink name=filesink
        "
        );

        let pipeline = gstreamer::parse::launch(&video)?
            .dynamic_cast::<Pipeline>()
            .expect("Failed to cast element to pipeline");
        let filesink = pipeline
            .by_name("filesink")
            .wrap_err("Failed to find 'filesink' element")?;
        filesink.set_property_from_str(
            "location",
            path.to_str().ok_or_eyre("Path must be valid UTF-8")?,
        );

        tracing::debug!("Created pipeline");

        Ok(pipeline.upcast::<gstreamer::Element>())
    }
}
