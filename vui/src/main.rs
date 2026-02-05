mod api;
mod audio;

use api::claude::Message as ChatMessage;
use iced::mouse;
use iced::time::{self, milliseconds};
use iced::widget::{canvas, center, column, container, text};
use iced::{Color, Element, Fill, Point, Rectangle, Renderer, Subscription, Task, Theme};
use std::sync::Arc;
use std::time::Instant;

fn main() -> iced::Result {
    dotenvy::dotenv().ok();

    iced::application(App::new, App::update, App::view)
        .subscription(App::subscription)
        .theme(App::theme)
        .run()
}

#[derive(Debug, Clone)]
enum Message {
    Tick,
    TtsReady(Result<Vec<u8>, String>),
    TtsFinished,
    TranscriptionReady(Result<String, String>),
    ClaudeReady(Result<String, String>),
    StartListening,
    StopListening,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
enum State {
    Idle,
    Speaking { text: String },
    Listening,
    Processing,
    Done,
}

struct App {
    state: State,
    start: Instant,
    cache: canvas::Cache,
    messages: Vec<ChatMessage>,
    subtitle: String,
    elevenlabs: Option<Arc<api::elevenlabs::Client>>,
    chat: Option<Arc<api::claude::Client>>,
    playback: Option<audio::Playback>,
    capture: Option<audio::Capture>,
    audio_buffer: Vec<f32>,
    silence_frames: usize,
}

impl App {
    fn new() -> (Self, Task<Message>) {
        let elevenlabs_key = std::env::var("ELEVENLABS_API_KEY").unwrap_or_default();
        let anthropic_key = std::env::var("ANTHROPIC_API_KEY").unwrap_or_default();

        if elevenlabs_key.is_empty() {
            eprintln!("WARNING: ELEVENLABS_API_KEY not set");
        }
        if anthropic_key.is_empty() {
            eprintln!("WARNING: ANTHROPIC_API_KEY not set");
        }

        let elevenlabs = if !elevenlabs_key.is_empty() {
            Some(Arc::new(api::elevenlabs::Client::new(elevenlabs_key)))
        } else {
            None
        };

        let chat = if !anthropic_key.is_empty() {
            Some(Arc::new(api::claude::Client::new(anthropic_key)))
        } else {
            None
        };

        let playback = audio::Playback::new().ok();

        let app = Self {
            state: State::Idle,
            start: Instant::now(),
            cache: canvas::Cache::default(),
            messages: Vec::new(),
            subtitle: "Say something to begin...".to_string(),
            elevenlabs,
            chat,
            playback,
            capture: None,
            audio_buffer: Vec::new(),
            silence_frames: 0,
        };

        // Start listening for the user
        (app, Task::done(Message::StartListening))
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Tick => {
                self.cache.clear();

                // Check playback status
                if let Some(ref playback) = self.playback
                    && let Some(status) = playback.try_recv_status()
                {
                    match status {
                        audio::playback::Status::Finished => {
                            return Task::done(Message::TtsFinished);
                        }
                        audio::playback::Status::Error(e) => {
                            eprintln!("Playback error: {}", e);
                            return Task::done(Message::TtsFinished);
                        }
                        _ => {}
                    }
                }

                // Check for audio input while listening
                if matches!(self.state, State::Listening)
                    && let Some(ref capture) = self.capture
                {
                    while let Some(samples) = capture.try_recv() {
                        let rms: f32 = (samples.iter().map(|s| s * s).sum::<f32>()
                            / samples.len() as f32)
                            .sqrt();

                        if rms < 0.01 {
                            self.silence_frames += 1;
                        } else {
                            self.silence_frames = 0;
                        }

                        self.audio_buffer.extend(samples);

                        // Stop after ~1.5 seconds of silence
                        if self.audio_buffer.len() > 16000 && self.silence_frames > 90 {
                            return Task::done(Message::StopListening);
                        }
                    }
                }

                Task::none()
            }

            Message::ClaudeReady(result) => match result {
                Ok(response) => {
                    // Check if conversation is complete
                    let is_complete = response.contains("APPLICATION_COMPLETE");
                    let display_text = response.replace("APPLICATION_COMPLETE", "").trim().to_string();

                    self.messages.push(ChatMessage::assistant(&display_text));
                    self.subtitle = display_text.clone();

                    if is_complete {
                        self.state = State::Done;
                    }

                    // Speak the response
                    if let Some(tts) = &self.elevenlabs {
                        let tts = tts.clone();
                        Task::perform(
                            async move { tts.synthesize(&display_text).await },
                            Message::TtsReady,
                        )
                    } else {
                        Task::none()
                    }
                }
                Err(e) => {
                    eprintln!("Claude error: {}", e);
                    self.subtitle = format!("Error: {}", e);
                    self.state = State::Done;
                    Task::none()
                }
            },

            Message::TtsReady(result) => match result {
                Ok(audio_data) => {
                    if let Some(ref playback) = self.playback {
                        playback.play(audio_data);
                        self.state = State::Speaking {
                            text: self.subtitle.clone(),
                        };
                    }
                    Task::none()
                }
                Err(e) => {
                    eprintln!("TTS error: {}", e);
                    self.subtitle = format!("Error: {}", e);
                    self.state = State::Done;
                    Task::none()
                }
            },

            Message::TtsFinished => {
                if matches!(self.state, State::Done) {
                    Task::none()
                } else {
                    Task::done(Message::StartListening)
                }
            }

            Message::StartListening => {
                self.state = State::Listening;
                self.subtitle = "Listening...".to_string();
                self.audio_buffer.clear();
                self.silence_frames = 0;
                self.capture = audio::Capture::new().ok();
                Task::none()
            }

            Message::StopListening => {
                self.capture = None;
                self.state = State::Processing;
                self.subtitle = "Processing...".to_string();

                let buffer = std::mem::take(&mut self.audio_buffer);
                let sample_rate = audio::Capture::sample_rate();

                if let Some(stt) = &self.elevenlabs {
                    let stt = stt.clone();
                    Task::perform(
                        async move { stt.transcribe(&buffer, sample_rate).await },
                        Message::TranscriptionReady,
                    )
                } else {
                    Task::done(Message::TranscriptionReady(Ok("test response".to_string())))
                }
            }

            Message::TranscriptionReady(result) => match result {
                Ok(transcript) => {
                    eprintln!("User said: {}", transcript);
                    self.messages.push(ChatMessage::user(&transcript));
                    self.subtitle = format!("You: {}", transcript);

                    if let Some(chat) = &self.chat {
                        let chat = chat.clone();
                        let messages = self.messages.clone();
                        Task::perform(
                            async move { chat.send(&messages).await },
                            Message::ClaudeReady,
                        )
                    } else {
                        Task::none()
                    }
                }
                Err(e) => {
                    eprintln!("Transcription error: {}", e);
                    self.subtitle = format!("Error: {}", e);
                    Task::done(Message::StartListening)
                }
            },
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let circle = canvas(self as &Self).width(Fill).height(Fill);

        let subtitle = text(&self.subtitle).size(24).color(Color::WHITE);

        let content = column![
            container(circle).width(400).height(400),
            container(subtitle).padding(20),
        ]
        .align_x(iced::Alignment::Center);

        center(content).into()
    }

    fn subscription(&self) -> Subscription<Message> {
        time::every(milliseconds(16)).map(|_| Message::Tick)
    }

    fn theme(&self) -> Theme {
        Theme::Dark
    }
}

impl canvas::Program<Message> for App {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        let circle = self.cache.draw(renderer, bounds.size(), |frame| {
            let center = frame.center();
            let t = self.start.elapsed().as_secs_f32();

            let pulse = match &self.state {
                State::Speaking { .. } => 25.0 * (t * 5.0).sin().abs(),
                State::Listening => 15.0 * (t * 3.0).sin().abs(),
                State::Processing => 10.0 * (t * 8.0).sin().abs(),
                State::Idle | State::Done => 5.0 * (t * 1.5).sin().abs(),
            };

            let base_radius = 80.0;
            let radius = base_radius + pulse;

            let color = match &self.state {
                State::Speaking { .. } => Color::from_rgb(0.2, 0.6, 1.0),
                State::Listening => Color::from_rgb(0.2, 0.8, 0.4),
                State::Processing => Color::from_rgb(1.0, 0.6, 0.2),
                State::Idle => Color::from_rgb(0.5, 0.5, 0.5),
                State::Done => Color::from_rgb(0.6, 0.3, 0.8),
            };

            let path = canvas::Path::circle(Point::new(center.x, center.y), radius);
            frame.fill(&path, color);

            let inner_radius = radius * 0.6;
            let inner_color = Color {
                a: 0.3,
                ..Color::WHITE
            };
            let inner_path = canvas::Path::circle(Point::new(center.x, center.y), inner_radius);
            frame.fill(&inner_path, inner_color);
        });

        vec![circle]
    }
}
