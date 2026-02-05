mod api;
mod audio;
mod form;
mod theme;

use api::claude;
use iced::Length::FillPortion;
use iced::time::{self, milliseconds};
use iced::widget::{bottom, canvas, column, container, right, row, scrollable, space, stack, text};
use iced::{
    Center, Color, Element, Fill, Point, Rectangle, Renderer, Subscription, Task, Theme, color,
    mouse, padding, system,
};
use std::sync::Arc;
use std::time::Instant;

/// RMS threshold for barge-in detection during TTS playback.
/// 5x normal speech threshold (0.01) to avoid TTS echo triggering false barge-ins.
const BARGE_IN_THRESHOLD: f32 = 0.05;

/// Consecutive audio chunks above threshold required to trigger barge-in.
/// ~5 chunks ≈ 80-160ms of sustained loud speech.
const BARGE_IN_FRAMES: usize = 5;

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
    ClaudeReady(Result<claude::SendResult, String>),
    SubmitForm,
    SubmitResult(Result<String, String>),
    StartListening,
    StopListening,
    ResumePlayback,
    ThemeChanged(iced::theme::Mode),
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
enum State {
    Idle,
    Speaking { text: String },
    BargedIn,
    Listening,
    Processing { pending_text: Option<String> },
    Submitted,
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
    form: form::Form,
    elevenlabs: Option<Arc<api::elevenlabs::Client>>,
    chat: Option<Arc<api::claude::Client>>,
    playback: Option<audio::Playback>,
    capture: Option<audio::Capture>,
    audio_buffer: Vec<f32>,
    audio_level: f32,
    playback_level: f32,
    silence_frames: usize,
    has_speech: bool,
    barge_in_frames: usize,
    tts_paused: bool,
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
        let capture = audio::Capture::new().ok();

        let app = Self {
            state: State::Idle,
            start: Instant::now(),
            cache: canvas::Cache::default(),
            messages: Vec::new(),
            transcript: Vec::new(),
            form: form::Form::default(),
            elevenlabs,
            chat,
            playback,
            capture,
            audio_buffer: Vec::new(),
            audio_level: 0.0,
            playback_level: 0.0,
            silence_frames: 0,
            has_speech: false,
            barge_in_frames: 0,
            tts_paused: false,
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
                                if matches!(self.state, State::Speaking { .. }) {
                                    return Task::done(Message::TtsFinished);
                                }
                                // TTS ended during barge-in flow — mark not resumable
                                self.tts_paused = false;
                            }
                            audio::playback::Status::Error(e) => {
                                log::error!("Playback error: {}", e);
                                self.playback_level = 0.0;
                                self.tts_paused = false;
                                return Task::done(Message::TtsFinished);
                            }
                            audio::playback::Status::Level(level) => {
                                let target = (level * 5.0).min(1.0);
                                self.playback_level = self.playback_level * 0.7 + target * 0.3;
                            }
                            audio::playback::Status::Paused => {
                                self.playback_level = 0.0;
                            }
                            _ => {}
                        }
                    }
                }

                // Barge-in detection during Speaking
                if matches!(self.state, State::Speaking { .. })
                    && let Some(ref capture) = self.capture
                {
                    while let Some(samples) = capture.try_recv() {
                        let rms: f32 = (samples.iter().map(|s| s * s).sum::<f32>()
                            / samples.len() as f32)
                            .sqrt();

                        if rms >= BARGE_IN_THRESHOLD {
                            self.barge_in_frames += 1;
                        } else {
                            self.barge_in_frames = 0;
                        }

                        if self.barge_in_frames >= BARGE_IN_FRAMES {
                            log::debug!("Barge-in detected, pausing TTS");
                            if let Some(ref playback) = self.playback {
                                playback.pause();
                            }
                            self.tts_paused = true;
                            self.state = State::BargedIn;
                            self.audio_buffer.clear();
                            self.has_speech = true;
                            self.silence_frames = 0;
                            self.barge_in_frames = 0;
                            break;
                        }
                    }
                }

                // Capture during BargedIn (same silence detection as Listening)
                if matches!(self.state, State::BargedIn)
                    && let Some(ref capture) = self.capture
                {
                    while let Some(samples) = capture.try_recv() {
                        let rms: f32 = (samples.iter().map(|s| s * s).sum::<f32>()
                            / samples.len() as f32)
                            .sqrt();

                        let target_level = (rms * 10.0).min(1.0);
                        self.audio_level = self.audio_level * 0.8 + target_level * 0.2;

                        if rms >= 0.01 {
                            self.has_speech = true;
                            self.silence_frames = 0;
                        } else {
                            self.silence_frames += 1;
                        }

                        self.audio_buffer.extend(samples);

                        if self.audio_buffer.len() > 16000 && self.silence_frames > 90 {
                            if self.has_speech {
                                return Task::done(Message::StopListening);
                            } else {
                                return Task::done(Message::ResumePlayback);
                            }
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
                Ok(result) => {
                    // Build the assistant content blocks for message history
                    let mut blocks = Vec::new();
                    if !result.text.is_empty() {
                        blocks.push(claude::ContentBlock::Text {
                            text: result.text.clone(),
                        });
                    }
                    for tu in &result.tool_uses {
                        blocks.push(claude::ContentBlock::ToolUse {
                            id: tu.id.clone(),
                            name: tu.name.clone(),
                            input: tu.input.clone(),
                        });
                    }
                    if !blocks.is_empty() {
                        self.messages
                            .push(claude::Message::assistant_blocks(blocks));
                    }

                    // Process tool uses if stop_reason is "tool_use"
                    if result.stop_reason == "tool_use" && !result.tool_uses.is_empty() {
                        let mut submit_requested = false;
                        for tu in &result.tool_uses {
                            if tu.name == "update_form" {
                                if let Ok(tool_input) =
                                    serde_json::from_value::<form::ToolInput>(tu.input.clone())
                                {
                                    let tool_result = self.form.apply(&tool_input);
                                    if tool_input.action == "submit" && self.form.is_ready() {
                                        submit_requested = true;
                                    }
                                    self.messages
                                        .push(claude::Message::tool_result(&tu.id, &tool_result));
                                } else {
                                    self.messages.push(claude::Message::tool_result(
                                        &tu.id,
                                        "Error: invalid tool input",
                                    ));
                                }
                            }
                        }

                        if submit_requested {
                            self.state = State::Done;
                            if !result.text.is_empty() {
                                self.transcript.push(Line {
                                    role: Role::Assistant,
                                    text: result.text.clone(),
                                });
                            }
                            return Task::done(Message::SubmitForm);
                        }

                        // Re-send to Claude to continue the conversation
                        if let Some(chat) = &self.chat {
                            let chat = chat.clone();
                            let messages = self.messages.clone();
                            let tools = vec![form::Form::tool_definition()];
                            return Task::perform(
                                async move { chat.send(&messages, &tools).await },
                                Message::ClaudeReady,
                            );
                        }
                        return Task::none();
                    }

                    // Normal end_turn — display text and TTS
                    let display_text = result.text.clone();

                    if display_text.is_empty() {
                        return Task::done(Message::StartListening);
                    }

                    self.state = State::Processing {
                        pending_text: Some(display_text.clone()),
                    };

                    if let Some(tts) = &self.elevenlabs {
                        let tts = tts.clone();
                        Task::perform(
                            async move { tts.synthesize(&display_text).await },
                            Message::TtsReady,
                        )
                    } else {
                        if let State::Processing {
                            ref mut pending_text,
                        } = self.state
                            && let Some(text) = pending_text.take()
                        {
                            self.transcript.push(Line {
                                role: Role::Assistant,
                                text,
                            });
                        }
                        Task::done(Message::StartListening)
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

            Message::SubmitForm => {
                log::info!("Form submitted: {:?}", self.form);
                println!("\n=== APPLICATION SUBMITTED ===\n{}\n", serde_json::to_string_pretty(&self.form.to_json()).unwrap());
                self.state = State::Submitted;
                let payload = self.form.to_json();
                let http = reqwest::Client::new();
                Task::perform(
                    async move {
                        http.post("https://hooks.attio.com/w/8b9f7dfe-0c44-4531-86e5-70f0ad1ec853/fffbdf49-989b-40bc-af97-fc602fd5bce9")
                            .json(&payload)
                            .send()
                            .await
                            .map_err(|e| e.to_string())?
                            .text()
                            .await
                            .map_err(|e| e.to_string())
                    },
                    Message::SubmitResult,
                )
            }

            Message::SubmitResult(result) => {
                match &result {
                    Ok(body) => log::info!("Submitted to Attio: {}", body),
                    Err(e) => log::error!("Attio webhook error: {}", e),
                }
                Task::none()
            }

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
                        // Don't start capture here — opening a CoreAudio input stream
                        // causes the audio graph to reconfigure and glitches playback.
                        // Capture is started lazily in the Tick handler after a short delay.
                        self.barge_in_frames = 0;
                        self.tts_paused = false;
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
                        && let Some(text) = pending_text.take()
                    {
                        self.transcript.push(Line {
                            role: Role::Assistant,
                            text,
                        });
                    }
                    self.state = State::Done;
                    Task::none()
                }
            },

            Message::TtsFinished => match self.state {
                State::Speaking { .. } => Task::done(Message::StartListening),
                _ => Task::none(),
            },

            Message::StartListening => {
                self.state = State::Listening;
                self.audio_buffer.clear();
                self.audio_level = 0.0;
                self.silence_frames = 0;
                self.has_speech = false;
                Task::none()
            }

            Message::StopListening => {
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

            Message::ResumePlayback => {
                if self.tts_paused {
                    log::debug!("Resuming TTS after noise barge-in");
                    if let Some(ref playback) = self.playback {
                        playback.resume();
                    }
                    self.state = State::Speaking {
                        text: String::new(),
                    };
                    self.audio_buffer.clear();
                    self.barge_in_frames = 0;
                    self.has_speech = false;
                    self.silence_frames = 0;
                    Task::none()
                } else {
                    // TTS finished while we were transcribing — nothing to resume
                    Task::done(Message::StartListening)
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
                        if self.tts_paused {
                            return Task::done(Message::ResumePlayback);
                        }
                        return Task::done(Message::StartListening);
                    }

                    // Real speech — if we barged in, stop TTS permanently
                    if self.tts_paused {
                        if let Some(ref playback) = self.playback {
                            playback.stop();
                        }
                        self.tts_paused = false;
                    }

                    self.messages.push(claude::Message::user(&transcript));
                    self.transcript.push(Line {
                        role: Role::User,
                        text: transcript,
                    });

                    if let Some(chat) = &self.chat {
                        let chat = chat.clone();
                        let messages = self.messages.clone();
                        let tools = vec![form::Form::tool_definition()];
                        Task::perform(
                            async move { chat.send(&messages, &tools).await },
                            Message::ClaudeReady,
                        )
                    } else {
                        Task::none()
                    }
                }
                Err(e) => {
                    log::debug!("Transcription error: {}", e);
                    if self.tts_paused {
                        return Task::done(Message::ResumePlayback);
                    }
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
            .map(|line| {
                let bubble = container(text(&line.text).size(16))
                    .padding(padding::all(10).left(14).right(14))
                    .max_width(500);

                match line.role {
                    Role::User => right(bubble.style(theme::user)).into(),
                    Role::Assistant => bubble.style(theme::assistant).into(),
                }
            })
            .collect();

        let transcript_view = if transcript_lines.is_empty() {
            bottom(text("Say something to begin...").size(16).color(dim_color)).center_x(Fill)
        } else {
            bottom(
                scrollable(
                    row![
                        space().width(FillPortion(1)),
                        column(transcript_lines)
                            .spacing(8)
                            .padding(10)
                            .width(FillPortion(4)),
                        space().width(FillPortion(1)),
                    ]
                    .width(Fill),
                )
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
            State::Listening | State::BargedIn => "Listening...",
            State::Processing { .. } => "Processing...",
            State::Speaking { .. } => "Speaking...",
            State::Submitted => "Submitted",
            State::Done => "Done",
            State::Idle => "",
        };

        let status_color = match &self.state {
            State::Listening | State::BargedIn => color!(0xcc3e28),
            _ => dim_color,
        };

        let voice_area = stack![
            bottom(transcript_view).width(Fill).padding(10),
            container(
                column![
                    container(circle).width(300).height(300),
                    container(text(status).size(14).color(status_color)).padding(5),
                ]
                .padding(padding::bottom(300))
                .spacing(10)
                .align_x(Center)
                .width(Fill)
            )
            .center_y(Fill)
            .style(theme::fade),
        ]
        .width(Fill);

        let sidebar = if matches!(self.state, State::Submitted) {
            self.form.view_submitted()
        } else {
            if self.form.is_empty() {
                return voice_area.into();
            }
            self.form
                .view(self.form.is_ready().then(|| Message::SubmitForm))
        };

        stack![
            row![voice_area, space().width(280)],
            right(container(sidebar).width(280))
        ]
        .into()
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

            // Processing draws a spinning arc instead of a filled circle
            if matches!(&self.state, State::Processing { .. }) {
                let radius = 40.0;
                let sweep = std::f32::consts::FRAC_PI_2; // 90° arc
                let start_angle = t * 4.0;
                let arc = canvas::path::Arc {
                    center: Point::new(center.x, center.y),
                    radius,
                    start_angle: iced::Radians(start_angle),
                    end_angle: iced::Radians(start_angle + sweep),
                };
                let path = canvas::Path::new(|b| b.arc(arc));
                frame.stroke(
                    &path,
                    canvas::Stroke::default()
                        .with_color(Color::from_rgb(1.0, 0.6, 0.2))
                        .with_width(4.0),
                );
            } else {
                let (base_radius, pulse, color) = match &self.state {
                    State::Speaking { .. } => {
                        // Blue circle reacts to actual playback volume
                        let audio_pulse = self.playback_level * 40.0;
                        let base_pulse = if self.playback_level > 0.05 {
                            10.0 * (t * 4.0).sin().abs()
                        } else {
                            0.0
                        };
                        (
                            60.0,
                            base_pulse + audio_pulse,
                            Color::from_rgb(0.2, 0.6, 1.0),
                        )
                    }
                    State::Listening => {
                        if self.has_speech {
                            // Reactive red circle
                            let audio_pulse = self.audio_level * 40.0;
                            let base_pulse = if self.audio_level > 0.05 {
                                10.0 * (t * 2.0).sin().abs()
                            } else {
                                0.0
                            };
                            (
                                60.0,
                                base_pulse + audio_pulse,
                                Color::from_rgb(0.85, 0.2, 0.2),
                            )
                        } else {
                            // Small idle dot
                            (12.0, 0.0, Color::from_rgb(0.85, 0.2, 0.2))
                        }
                    }
                    State::BargedIn => {
                        let audio_pulse = self.audio_level * 40.0;
                        let base_pulse = if self.audio_level > 0.05 {
                            10.0 * (t * 2.0).sin().abs()
                        } else {
                            0.0
                        };
                        (
                            60.0,
                            base_pulse + audio_pulse,
                            Color::from_rgb(0.85, 0.2, 0.2),
                        )
                    }
                    State::Submitted => (
                        60.0,
                        3.0 * (t * 1.0).sin().abs(),
                        Color::from_rgb(0.2, 0.65, 0.3),
                    ),
                    State::Idle | State::Done => (
                        60.0,
                        5.0 * (t * 1.5).sin().abs(),
                        Color::from_rgb(0.5, 0.5, 0.5),
                    ),
                    State::Processing { .. } => unreachable!(),
                };

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
            }
        });

        vec![circle]
    }
}
