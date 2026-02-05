use rodio::{Decoder, OutputStream, Sink, Source};
use std::io::Cursor;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::mpsc;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

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
    Level(f32),
    Finished,
    Error(String),
}

/// A source wrapper that computes RMS levels as f32 audio plays through
struct Level<S: Source<Item = f32>> {
    source: S,
    level: Arc<AtomicU32>,
    buffer: Vec<f32>,
}

impl<S: Source<Item = f32>> Level<S> {
    fn new(source: S, level: Arc<AtomicU32>) -> Self {
        Self {
            source,
            level,
            buffer: Vec::with_capacity(1024),
        }
    }
}

impl<S: Source<Item = f32>> Iterator for Level<S> {
    type Item = f32;

    fn next(&mut self) -> Option<f32> {
        let sample = self.source.next()?;

        self.buffer.push(sample);

        if self.buffer.len() >= 1024 {
            let rms = (self.buffer.iter().map(|s| s * s).sum::<f32>()
                / self.buffer.len() as f32)
                .sqrt();
            self.level.store(rms.to_bits(), Ordering::Relaxed);
            self.buffer.clear();
        }

        Some(sample)
    }
}

impl<S: Source<Item = f32>> Source for Level<S> {
    fn current_frame_len(&self) -> Option<usize> {
        self.source.current_frame_len()
    }

    fn channels(&self) -> u16 {
        self.source.channels()
    }

    fn sample_rate(&self) -> u32 {
        self.source.sample_rate()
    }

    fn total_duration(&self) -> Option<Duration> {
        self.source.total_duration()
    }
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

                                let level = Arc::new(AtomicU32::new(0));
                                let level_clone = level.clone();

                                // Convert to f32 and wrap with level tracker
                                let f32_source = source.convert_samples::<f32>();
                                let tracked_source = Level::new(f32_source, level);

                                sink.append(tracked_source);

                                // Poll level while playing
                                while !sink.empty() {
                                    let bits = level_clone.load(Ordering::Relaxed);
                                    let rms = f32::from_bits(bits);
                                    let _ = status_sender.send(Status::Level(rms));
                                    thread::sleep(Duration::from_millis(16));
                                }

                                let _ = status_sender.send(Status::Level(0.0));
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
