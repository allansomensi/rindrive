use crate::{message::Message, state::AppState, view, worker};
use iced::{Subscription, Task};
use rfd::FileDialog;
use rindrive_core::engine::EngineType;
use rindrive_i18n::fl;
use std::{
    path::PathBuf,
    sync::{Arc, atomic::AtomicBool},
};

pub struct App {
    pub state: AppState,
    pub selected_drive: Option<PathBuf>,
    pub log: String,
    pub progress: f32,
    pub sections_input: String,
    pub buffer_size_input: String,
    pub selected_engine: EngineType,
    pub block_map: Vec<u8>,
    pub cancel_flag: Option<Arc<AtomicBool>>,
}

impl Default for App {
    fn default() -> Self {
        Self {
            state: AppState::Waiting,
            selected_drive: None,
            log: fl!("log-waiting"),
            progress: 0.0,

            // Defaults
            sections_input: "600".to_string(),
            buffer_size_input: "4096".to_string(),
            selected_engine: EngineType::SpotCheck,
            block_map: vec![0; 600],
            cancel_flag: None,
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
                    if val.is_empty() {
                        self.sections_input = val;
                        self.block_map = vec![];
                    } else if let Ok(n) = val.parse::<usize>()
                        && n <= 2000
                    {
                        self.sections_input = val;
                        self.block_map = vec![0; n];
                    }
                }
                Task::none()
            }

            Message::BufferSizeChanged(val) => {
                if val.chars().all(|c| c.is_numeric()) && val.is_empty() {
                    self.buffer_size_input = val;
                } else if let Ok(n) = val.parse::<usize>()
                    && n <= 268_435_456
                {
                    self.buffer_size_input = val;
                }

                Task::none()
            }

            Message::EngineSelected(engine) => {
                if !matches!(self.state, AppState::Auditing | AppState::Cancelling) {
                    self.selected_engine = engine;
                }
                Task::none()
            }

            Message::SelectManual => {
                Task::perform(async { FileDialog::new().pick_folder() }, |opt| {
                    opt.map(Message::DriveDetected)
                        .unwrap_or(Message::Progress(0.0, fl!("log-cancelled")))
                })
            }

            Message::DriveDetected(path) => {
                self.selected_drive = Some(path.clone());
                self.log = fl!("log-detected", path = path.display().to_string());
                self.state = AppState::Ready;
                self.progress = 0.0;

                if let Ok(n) = self.sections_input.parse::<usize>() {
                    self.block_map = vec![0; n];
                }
                Task::none()
            }

            Message::StartAudit => {
                if let Ok(buf_size) = self.buffer_size_input.parse::<usize>() {
                    if buf_size < 512 {
                        self.buffer_size_input = "512".to_string();
                    }
                } else {
                    self.buffer_size_input = "1048576".to_string();
                }

                if let Some(path) = self.selected_drive.clone() {
                    self.state = AppState::Auditing;
                    self.log = fl!("log-initializing").to_string();
                    self.progress = 0.0;
                    self.block_map.fill(0);

                    let flag = Arc::new(AtomicBool::new(false));
                    self.cancel_flag = Some(flag.clone());

                    let sections = self.sections_input.parse::<usize>().unwrap_or(600);
                    let buffer = self.buffer_size_input.parse::<usize>().unwrap_or(4096);

                    Task::run(worker::audit::run(path, sections, buffer, flag), |evt| evt)
                } else {
                    Task::none()
                }
            }

            Message::CancelAudit => {
                if let Some(flag) = &self.cancel_flag {
                    flag.store(true, std::sync::atomic::Ordering::Relaxed);
                }

                self.state = AppState::Cancelling;
                self.log = fl!("log-cancelling").to_string();

                Task::none()
            }

            Message::Progress(pct, msg) => {
                self.progress = pct * 100.0;
                self.log = msg;
                Task::none()
            }

            Message::BlockUpdated(idx, status) => {
                if idx < self.block_map.len() {
                    self.block_map[idx] = status;
                }
                Task::none()
            }

            Message::Finished(result) => {
                if self.state == AppState::Cancelling {
                    self.state = AppState::Ready;
                    self.log = fl!("log-cancelled").to_string();
                    self.progress = 0.0;

                    if let Ok(n) = self.sections_input.parse::<usize>() {
                        self.block_map = vec![0; n];
                    }
                    return Task::none();
                }

                self.state = AppState::Finished;
                self.progress = 100.0;

                match result {
                    Ok(report) => {
                        self.log = if report.has_errors {
                            fl!("log-fake").to_string()
                        } else {
                            fl!("log-genuine").to_string()
                        };
                    }
                    Err(e) => {
                        self.log = fl!("log-error", error = e.to_string());
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
