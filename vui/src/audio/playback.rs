use rodio::{Decoder, OutputStream, Sink};
use std::io::Cursor;
use std::sync::mpsc;
use std::thread;

pub struct Playback {
    sender: mpsc::Sender<Command>,
    status_receiver: mpsc::Receiver<Status>,
}

#[allow(dead_code)]
enum Command {
    Play(Vec<u8>),
    Stop,
}

#[derive(Debug, Clone)]
pub enum Status {
    Playing,
    Finished,
    Error(String),
}

impl Playback {
    pub fn new() -> Result<Self, String> {
        let (sender, receiver) = mpsc::channel::<Command>();
        let (status_sender, status_receiver) = mpsc::channel();

        thread::spawn(move || {
            let (_stream, stream_handle) = match OutputStream::try_default() {
                Ok(s) => s,
                Err(e) => {
                    let _ = status_sender.send(Status::Error(e.to_string()));
                    return;
                }
            };

            let sink = Sink::try_new(&stream_handle).unwrap();

            while let Ok(cmd) = receiver.recv() {
                match cmd {
                    Command::Play(data) => {
                        let cursor = Cursor::new(data);
                        match Decoder::new(cursor) {
                            Ok(source) => {
                                let _ = status_sender.send(Status::Playing);
                                sink.append(source);
                                sink.sleep_until_end();
                                let _ = status_sender.send(Status::Finished);
                            }
                            Err(e) => {
                                let _ = status_sender.send(Status::Error(e.to_string()));
                            }
                        }
                    }
                    Command::Stop => {
                        sink.stop();
                    }
                }
            }
        });

        Ok(Self {
            sender,
            status_receiver,
        })
    }

    pub fn play(&self, audio_data: Vec<u8>) {
        let _ = self.sender.send(Command::Play(audio_data));
    }

    #[allow(dead_code)]
    pub fn stop(&self) {
        let _ = self.sender.send(Command::Stop);
    }

    pub fn try_recv_status(&self) -> Option<Status> {
        self.status_receiver.try_recv().ok()
    }
}
