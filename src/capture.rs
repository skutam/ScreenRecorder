use std::time::{Duration, Instant};
use windows_capture::capture::{Context, GraphicsCaptureApiHandler};
use windows_capture::frame::Frame;
use windows_capture::graphics_capture_api::InternalCaptureControl;
use crate::DefaultError;

pub struct Capture {
    frames: u64,
    last_report: Instant,
}

impl GraphicsCaptureApiHandler for Capture {
    type Flags = ();
    type Error = DefaultError;

    fn new(ctx: Context<Self::Flags>) -> Result<Self, Self::Error> {
        Ok(Self {
            frames: 0,
            last_report: Instant::now(),
        })
    }

    fn on_frame_arrived(&mut self, frame: &mut Frame, capture_control: InternalCaptureControl) -> Result<(), Self::Error> {
        self.frames += 1;

        if self.frames == 1 || self.last_report.elapsed() > Duration::from_secs(1) {
            println!(
                "Prijate snimky: {}, rozmery: {} x {}",
                self.frames,
                frame.width(),
                frame.height()
            );
            self.last_report = Instant::now();
        }

        Ok(())
    }

    fn on_closed(&mut self) -> Result<(), Self::Error> {
        println!("Snimanie ukoncene");
        Ok(())
    }
}