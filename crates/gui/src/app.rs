use crate::{message::Message, state::AppState, update, view, worker};
use iced::{Subscription, Task, widget::canvas::Cache};
use rindrive_core::engine::EngineType;
use rindrive_i18n::fl;
use std::{
    path::PathBuf,
    sync::{Arc, atomic::AtomicBool},
};

pub const DEFAULT_SECTIONS: usize = 600;
pub const MAX_SECTIONS: usize = 2000;
pub const FULLSCAN_BLOCKS: usize = 100;
pub const DEFAULT_BUFFER_SIZE: usize = 4096;
pub const MIN_BUFFER_SIZE: usize = 512;
pub const MAX_BUFFER_SIZE: usize = 268_435_456;

#[derive(Debug, Clone)]
pub struct UsbHardwareInfo {
    pub manufacturer: Option<String>,
    pub product: Option<String>,
    pub speed: Option<nusb::Speed>,
}

#[derive(Debug, Clone)]
pub struct DriveInfoResult {
    pub actual_path: PathBuf,
    pub drive_name: String,
    pub drive_capacity: String,
    pub usb_info: Option<UsbHardwareInfo>,
}

pub struct App {
    pub state: AppState,
    pub selected_drive: Option<PathBuf>,
    pub drive_name: String,
    pub drive_capacity: String,
    pub log: String,
    pub progress: f32,
    pub sections_input: String,
    pub buffer_size_input: String,
    pub selected_engine: EngineType,
    pub block_map: Vec<u8>,
    pub map_cache: Cache,
    pub cancel_flag: Option<Arc<AtomicBool>>,
    pub usb_info: Option<UsbHardwareInfo>,
    pub last_report: Option<Arc<rindrive_core::engine::spotcheck::Report>>,
    pub audit_time: Option<String>,
}

impl Default for App {
    fn default() -> Self {
        Self {
            state: AppState::Waiting,
            selected_drive: None,
            drive_name: String::new(),
            drive_capacity: String::new(),
            log: fl!("log-waiting"),
            progress: 0.0,
            sections_input: DEFAULT_SECTIONS.to_string(),
            buffer_size_input: DEFAULT_BUFFER_SIZE.to_string(),
            selected_engine: EngineType::SpotCheck,
            block_map: vec![0; DEFAULT_SECTIONS],
            map_cache: Cache::default(),
            cancel_flag: None,
            usb_info: None,
            last_report: None,
            audit_time: None,
        }
    }
}

impl App {
    pub fn new() -> (Self, Task<Message>) {
        (Self::default(), Task::none())
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        update::handle_message(self, message)
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

    pub(crate) fn is_idle(&self) -> bool {
        !matches!(self.state, AppState::Auditing | AppState::Cancelling)
    }

    pub(crate) fn rebuild_block_map(&mut self) {
        match self.selected_engine {
            EngineType::SpotCheck => {
                let n = self
                    .sections_input
                    .parse::<usize>()
                    .unwrap_or(DEFAULT_SECTIONS);
                self.block_map = vec![0; n];
            }
            EngineType::FullScan => {
                self.block_map = vec![0; FULLSCAN_BLOCKS];
            }
        }
        self.map_cache.clear();
    }

    pub(crate) fn reset_audit_state(&mut self, log_message: String) {
        self.state = AppState::Waiting;
        self.progress = 0.0;
        self.log = log_message;
        self.last_report = None;
        self.audit_time = None;
        self.cancel_flag = None;
        self.rebuild_block_map();
    }

    pub(crate) fn validate_buffer_size(&mut self) {
        if let Ok(buf_size) = self.buffer_size_input.parse::<usize>() {
            if buf_size < MIN_BUFFER_SIZE {
                self.buffer_size_input = MIN_BUFFER_SIZE.to_string();
            }
        } else {
            self.buffer_size_input = DEFAULT_BUFFER_SIZE.to_string();
        }
    }
}
