use crate::{message::Message, state::AppState, view, worker};
use iced::{Subscription, Task};
use rfd::FileDialog;
use rindrive_core::engine::EngineType;
use std::path::PathBuf;

pub struct App {
    pub state: AppState,
    pub selected_drive: Option<PathBuf>,
    pub log: String,
    pub progress: f32,
    pub sections_input: String,
    pub buffer_size_input: String,
    pub selected_engine: EngineType,
    pub block_map: Vec<u8>,
}

impl Default for App {
    fn default() -> Self {
        Self {
            state: AppState::Waiting,
            selected_drive: None,
            log: "Waiting for USB drive...".to_string(),
            progress: 0.0,

            // Defaults
            sections_input: "576".to_string(),
            buffer_size_input: "4096".to_string(),
            selected_engine: EngineType::SpotCheck,
            block_map: vec![0; 576],
        }
    }
}

impl App {
    pub fn new() -> (Self, Task<Message>) {
        (Self::default(), Task::none())
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::SectionsChanged(val) => {
                if val.chars().all(|c| c.is_numeric()) {
                    self.sections_input = val;

                    if let Ok(n) = self.sections_input.parse::<usize>() {
                        self.block_map = vec![0; n];
                    }
                }
                Task::none()
            }
            Message::BufferSizeChanged(val) => {
                if val.chars().all(|c| c.is_numeric()) {
                    self.buffer_size_input = val;
                }
                Task::none()
            }
            Message::EngineSelected(engine) => {
                self.selected_engine = engine;
                Task::none()
            }

            Message::SelectManual => {
                Task::perform(async { FileDialog::new().pick_folder() }, |opt| {
                    opt.map(Message::DriveDetected)
                        .unwrap_or(Message::Progress(0.0, "Selection cancelled".to_string()))
                })
            }

            Message::DriveDetected(path) => {
                self.selected_drive = Some(path.clone());
                self.log = format!("Detected: {}", path.display());
                self.state = AppState::Ready;
                self.progress = 0.0;

                if let Ok(n) = self.sections_input.parse::<usize>() {
                    self.block_map = vec![0; n];
                }
                Task::none()
            }

            Message::StartAudit => {
                if let Some(path) = self.selected_drive.clone() {
                    self.state = AppState::Auditing;
                    self.log = "Initializing engine...".to_string();
                    self.progress = 0.0;

                    Task::run(worker::audit::run(path), |evt| evt)
                } else {
                    Task::none()
                }
            }

            Message::Progress(pct, msg) => {
                self.progress = pct * 100.0;
                self.log = msg;

                let total_blocks = self.block_map.len();
                let current_index = (self.progress / 100.0 * total_blocks as f32) as usize;

                for i in 0..current_index.min(total_blocks) {
                    if self.block_map[i] == 0 {
                        self.block_map[i] = 1;
                    }
                }

                Task::none()
            }

            Message::Finished(result) => {
                self.state = AppState::Finished;
                self.progress = 100.0;

                match result {
                    Ok(report) => {
                        let status = if report.has_errors { 2 } else { 1 };
                        self.block_map.fill(status);
                        self.log = if report.has_errors {
                            "❌ FAKE / CORRUPTED!".to_string()
                        } else {
                            "✅ GENUINE (100% OK)".to_string()
                        };
                    }
                    Err(e) => {
                        self.log = format!("Critical Error: {e}");
                        self.block_map.fill(2);
                    }
                };
                Task::none()
            }
        }
    }

    pub fn subscription(&self) -> Subscription<Message> {
        if self.state != AppState::Auditing {
            Subscription::run(worker::usb::stream)
        } else {
            Subscription::none()
        }
    }

    pub fn view(&self) -> iced::Element<'_, Message> {
        view::view(self)
    }
}
