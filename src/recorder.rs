use std::sync::mpsc::{self, SendError, Sender};
use std::thread;
use std::thread::JoinHandle;
use windows_capture::capture::{CaptureControl, GraphicsCaptureApiHandler};
use windows_capture::monitor::Monitor;
use windows_capture::settings::{ColorFormat, CursorCaptureSettings, DirtyRegionSettings, DrawBorderSettings, MinimumUpdateIntervalSettings, SecondaryWindowSettings, Settings};
use crate::capture::Capture;
use crate::DefaultError;

pub enum RecorderCommand {
    Start,
    Stop,
    Shutdown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecorderState {
    Idle,
    Recording,
    Error,
}

pub type RecorderError = SendError<RecorderCommand>;

pub struct Recorder {
    command_sender: Sender<RecorderCommand>,
    worker: Option<JoinHandle<()>>,
}

impl Recorder {
    pub fn new(notify: impl Fn(RecorderState) + Send + 'static) -> Recorder {
        let (sender, receiver) = mpsc::channel::<RecorderCommand>();
        let mut capture: Option<CaptureControl<Capture, DefaultError>> = None;

        Recorder {
            worker: Some(thread::spawn(move || {
                let mut state = RecorderState::Idle;
                notify(state);
                while let Ok(command) = receiver.recv() {
                    match command {
                        RecorderCommand::Start => {
                            if state == RecorderState::Idle {
                                match new_capture() {
                                    Ok(new_capture) => {
                                        capture = Some(new_capture);
                                        state = RecorderState::Recording;
                                        notify(state);
                                        println!("Recorder started");
                                    }
                                    Err(error) => {
                                        state = RecorderState::Error;
                                        notify(state);
                                        eprintln!("Failed to create new capture: {:?}", error)
                                    },
                                }
                            } else {
                                println!("Recorder already started, ignoring");
                            }
                        },
                        RecorderCommand::Stop => {
                            if state == RecorderState::Recording {
                                if let Some(capture) = capture.take() {
                                    if let Err(error) = capture.stop() {
                                        state = RecorderState::Error;
                                        notify(state);
                                        eprintln!("Failed to shut down: {:?}", error);
                                        continue;
                                    }
                                }
                                state = RecorderState::Idle;
                                notify(state);
                                println!("Recorder stopped");
                            } else {
                                println!("Recorder already stopped, ignoring");
                            }
                        },
                        RecorderCommand::Shutdown => {
                            if let Some(capture) = capture.take() {
                                if let Err(error) = capture.stop() {
                                    eprintln!("Failed to shut down: {:?}", error);
                                    state = RecorderState::Error;
                                    notify(state);
                                }
                            }
                            break
                        },
                    }
                }
            })),
            command_sender: sender,
        }
    }

    pub fn start(&self) -> Result<(), RecorderError> {
        self.command_sender.send(RecorderCommand::Start)
    }

    pub fn stop(&self) -> Result<(), RecorderError> {
        self.command_sender.send(RecorderCommand::Stop)
    }

    pub fn shutdown(&mut self) -> Result<(), crate::DefaultError> {
        let Some(worker) = self.worker.take() else {
            return Ok(());
        };

        let send_result = self.command_sender.send(RecorderCommand::Shutdown);

        worker.join().map_err(|_| "Recorder thread panicked")?;
        send_result?;
        Ok(())
    }
}

fn new_capture() -> Result<CaptureControl<Capture, DefaultError>, DefaultError> {
    let settings = Settings::new(
        Monitor::primary()?,
        CursorCaptureSettings::Default,
        DrawBorderSettings::Default,
        SecondaryWindowSettings::Default,
        MinimumUpdateIntervalSettings::Default,
        DirtyRegionSettings::Default,
        ColorFormat::Bgra8,
        (),
    );

    Ok(Capture::start_free_threaded(settings)?)
}