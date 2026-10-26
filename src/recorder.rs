use std::sync::mpsc::{self, SendError, Sender};
use std::thread;
use std::thread::JoinHandle;

pub enum RecorderCommand {
    Start,
    Stop,
    Shutdown,
}

pub type RecorderError = SendError<RecorderCommand>;

pub struct Recorder {
    command_sender: Sender<RecorderCommand>,
    worker: Option<JoinHandle<()>>,
}

impl Recorder {
    pub fn new() -> Recorder {
        let (sender, receiver) = mpsc::channel::<RecorderCommand>();

        Recorder {
            worker: Some(thread::spawn(move || {
                while let Ok(command) = receiver.recv() {
                    match command {
                        RecorderCommand::Start => {
                            println!("Recorder started");
                        },
                        RecorderCommand::Stop => {
                            println!("Recorder stopped");
                        },
                        RecorderCommand::Shutdown => {
                            break;
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