mod api;
mod audio;
mod theme;

use api::claude;
use iced::time::{self, milliseconds};
use iced::widget::{bottom, canvas, center, column, container, scrollable, stack, text};
use iced::{
    Center, Color, Element, Fill, Point, Rectangle, Renderer, Subscription, Task, Theme, color,
    mouse, padding, system,
};
use std::sync::Arc;
use std::time::Instant;

fn main() -> iced::Result {
    dotenvy::dotenv().ok();
    env_logger::init();

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
    ThemeChanged(iced::theme::Mode),
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
enum State {
    Idle,
    Speaking { text: String },
    Listening,
    Processing { pending_text: Option<String> },
    Done,
}

#[derive(Debug, Clone)]
enum Role {
    User,
    Assistant,
}

#[derive(Debug, Clone)]
struct Line {
    role: Role,
    text: String,
}

struct App {
    state: State,
    start: Instant,
    cache: canvas::Cache,
    messages: Vec<claude::Message>,
    transcript: Vec<Line>,
    elevenlabs: Option<Arc<api::elevenlabs::Client>>,
    chat: Option<Arc<api::claude::Client>>,
    playback: Option<audio::Playback>,
    capture: Option<audio::Capture>,
    audio_buffer: Vec<f32>,
    audio_level: f32,
    playback_level: f32,
    silence_frames: usize,
    has_speech: bool,
    theme_mode: iced::theme::Mode,
}

impl App {
    fn new() -> (Self, Task<Message>) {
        let elevenlabs_key = std::env::var("ELEVENLABS_API_KEY").unwrap_or_default();
        let anthropic_key = std::env::var("ANTHROPIC_API_KEY").unwrap_or_default();

        if elevenlabs_key.is_empty() {
            log::error!("WARNING: ELEVENLABS_API_KEY not set");
        }
        if anthropic_key.is_empty() {
            log::error!("WARNING: ANTHROPIC_API_KEY not set");
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
            transcript: Vec::new(),
            elevenlabs,
            chat,
            playback,
            capture: None,
            audio_buffer: Vec::new(),
            audio_level: 0.0,
            playback_level: 0.0,
            silence_frames: 0,
            has_speech: false,
            theme_mode: iced::theme::Mode::Dark,
        };

        let task = Task::batch([
            Task::done(Message::StartListening),
            system::theme().map(Message::ThemeChanged),
        ]);

        (app, task)
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Tick => {
                self.cache.clear();

                // Check playback status (drain all pending messages)
                if let Some(ref playback) = self.playback {
                    while let Some(status) = playback.try_recv_status() {
                        match status {
                            audio::playback::Status::Finished => {
                                self.playback_level = 0.0;
                                return Task::done(Message::TtsFinished);
                            }
                            audio::playback::Status::Error(e) => {
                                log::error!("Playback error: {}", e);
                                self.playback_level = 0.0;
                                return Task::done(Message::TtsFinished);
                            }
                            audio::playback::Status::Level(level) => {
                                let target = (level * 5.0).min(1.0);
                                self.playback_level = self.playback_level * 0.7 + target * 0.3;
                            }
                            _ => {}
                        }
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

                        // Smooth audio level for visual display
                        let target_level = (rms * 10.0).min(1.0);
                        self.audio_level = self.audio_level * 0.8 + target_level * 0.2;

                        if rms >= 0.01 {
                            self.has_speech = true;
                            self.silence_frames = 0;
                        } else {
                            self.silence_frames += 1;
                        }

                        self.audio_buffer.extend(samples);

                        // Stop after ~1.5 seconds of silence
                        if self.audio_buffer.len() > 16000 && self.silence_frames > 90 {
                            if self.has_speech {
                                return Task::done(Message::StopListening);
                            } else {
                                // No speech detected, just reset and keep listening
                                self.audio_buffer.clear();
                                self.silence_frames = 0;
                            }
                        }
                    }
                }

                Task::none()
            }

            Message::ThemeChanged(mode) => {
                self.theme_mode = mode;
                Task::none()
            }

            Message::ClaudeReady(result) => match result {
                Ok(response) => {
                    let is_complete = response.contains("APPLICATION_COMPLETE");
                    let display_text = response
                        .replace("APPLICATION_COMPLETE", "")
                        .trim()
                        .to_string();

                    self.messages
                        .push(claude::Message::assistant(&display_text));

                    if is_complete {
                        // Show text immediately when done since there's no next interaction
                        self.transcript.push(Line {
                            role: Role::Assistant,
                            text: display_text.clone(),
                        });
                        self.state = State::Done;
                    } else {
                        self.state = State::Processing {
                            pending_text: Some(display_text.clone()),
                        };
                    }

                    if let Some(tts) = &self.elevenlabs {
                        let tts = tts.clone();
                        Task::perform(
                            async move { tts.synthesize(&display_text).await },
                            Message::TtsReady,
                        )
                    } else {
                        // No TTS — show text immediately as fallback
                        if !is_complete {
                            if let State::Processing {
                                ref mut pending_text,
                            } = self.state
                            {
                                if let Some(text) = pending_text.take() {
                                    self.transcript.push(Line {
                                        role: Role::Assistant,
                                        text,
                                    });
                                }
                            }
                        }
                        Task::none()
                    }
                }
                Err(e) => {
                    log::error!("Claude error: {}", e);
                    self.transcript.push(Line {
                        role: Role::Assistant,
                        text: format!("Error: {}", e),
                    });
                    self.state = State::Done;
                    Task::none()
                }
            },

            Message::TtsReady(result) => match result {
                Ok(audio_data) => {
                    // Reveal the assistant's response now that audio is ready
                    let response_text = if let State::Processing {
                        ref mut pending_text,
                    } = self.state
                    {
                        pending_text.take()
                    } else {
                        None
                    };

                    if let Some(text) = &response_text {
                        self.transcript.push(Line {
                            role: Role::Assistant,
                            text: text.clone(),
                        });
                    }

                    if let Some(ref playback) = self.playback {
                        playback.play(audio_data);
                        self.state = State::Speaking {
                            text: response_text.unwrap_or_default(),
                        };
                    }
                    Task::none()
                }
                Err(e) => {
                    log::error!("TTS error: {}", e);
                    // TTS failed — show pending text as fallback
                    if let State::Processing {
                        ref mut pending_text,
                    } = self.state
                    {
                        if let Some(text) = pending_text.take() {
                            self.transcript.push(Line {
                                role: Role::Assistant,
                                text,
                            });
                        }
                    }
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
                self.audio_buffer.clear();
                self.audio_level = 0.0;
                self.silence_frames = 0;
                self.has_speech = false;
                self.capture = audio::Capture::new().ok();
                Task::none()
            }

            Message::StopListening => {
                self.capture = None;
                self.state = State::Processing { pending_text: None };
                self.audio_level = 0.0;

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
                    let transcript = transcript.trim().to_string();
                    log::debug!("User said: {}", transcript);

                    // Filter out noise/static
                    let is_noise = transcript.is_empty()
                        || transcript.len() < 3
                        || transcript.starts_with('(')
                        || transcript.to_lowercase().contains("static");

                    if is_noise {
                        log::debug!("Filtered noise: {:?}", transcript);
                        return Task::done(Message::StartListening);
                    }

                    self.messages.push(claude::Message::user(&transcript));
                    self.transcript.push(Line {
                        role: Role::User,
                        text: transcript,
                    });

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
                    log::debug!("Transcription error: {}", e);
                    self.transcript.push(Line {
                        role: Role::Assistant,
                        text: format!("Error: {}", e),
                    });
                    Task::done(Message::StartListening)
                }
            },
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let circle = canvas(self as &Self).width(Fill).height(Fill);

        let text_color = self.theme().palette().text;
        let dim_color = Color {
            a: 0.5,
            ..text_color
        };

        let transcript_lines: Vec<Element<'_, Message>> = self
            .transcript
            .iter()
            .enumerate()
            .map(|(i, line)| {
                let is_latest = i == self.transcript.len() - 1;
                let color = if is_latest { text_color } else { dim_color };

                let prefix = match line.role {
                    Role::User => "You: ",
                    Role::Assistant => "",
                };

                text(format!("{}{}", prefix, line.text))
                    .size(18)
                    .color(color)
                    .into()
            })
            .collect();

        let transcript_view = if transcript_lines.is_empty() {
            bottom(text("Say something to begin...").size(18).color(dim_color)).center_x(Fill)
        } else {
            bottom(
                scrollable(column(transcript_lines).spacing(8).padding(10))
                    .spacing(0)
                    .direction(scrollable::Direction::Vertical(
                        scrollable::Scrollbar::new()
                            .width(1)
                            .margin(1)
                            .scroller_width(3),
                    ))
                    .anchor_bottom(),
            )
        };

        // Status indicator
        let status = match &self.state {
            State::Listening => "Listening...",
            State::Processing { .. } => "Processing...",
            State::Speaking { .. } => "Speaking...",
            State::Done => "Done",
            State::Idle => "",
        };

        let status_color = match &self.state {
            State::Listening => color!(0xcc3e28), // Red for recording
            _ => dim_color,
        };

        let content = stack![
            container(
                column![
                    container(circle).width(300).height(300),
                    container(text(status).size(14).color(status_color)).padding(5),
                ]
                .padding(padding::bottom(300))
                .spacing(10)
                .align_x(Center)
            )
            .center_y(Fill),
            bottom(transcript_view).width(600).padding(10),
        ];

        center(content).into()
    }

    fn subscription(&self) -> Subscription<Message> {
        Subscription::batch([
            time::every(milliseconds(16)).map(|_| Message::Tick),
            system::theme_changes().map(Message::ThemeChanged),
        ])
    }

    fn theme(&self) -> Theme {
        match self.theme_mode {
            iced::theme::Mode::Light => theme::paper(),
            _ => theme::paper_dark(),
        }
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

            let (pulse, color) = match &self.state {
                State::Speaking { .. } => {
                    // Blue circle reacts to actual playback volume
                    let audio_pulse = self.playback_level * 40.0;
                    let base_pulse = if self.playback_level > 0.05 {
                        10.0 * (t * 4.0).sin().abs()
                    } else {
                        0.0
                    };
                    (base_pulse + audio_pulse, Color::from_rgb(0.2, 0.6, 1.0))
                }
                State::Listening => {
                    // Red, still when silent, pulses with voice
                    let audio_pulse = self.audio_level * 40.0;
                    let base_pulse = if self.audio_level > 0.05 {
                        10.0 * (t * 2.0).sin().abs()
                    } else {
                        0.0
                    };
                    (base_pulse + audio_pulse, Color::from_rgb(0.85, 0.2, 0.2))
                }
                State::Processing { .. } => (
                    10.0 * (t * 8.0).sin().abs(),
                    Color::from_rgb(1.0, 0.6, 0.2), // Orange
                ),
                State::Idle | State::Done => (
                    5.0 * (t * 1.5).sin().abs(),
                    Color::from_rgb(0.5, 0.5, 0.5), // Gray
                ),
            };

            let base_radius = 60.0;
            let radius = base_radius + pulse;

            let path = canvas::Path::circle(Point::new(center.x, center.y), radius);
            frame.fill(&path, color);

            // Inner glow
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
