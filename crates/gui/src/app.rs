use crate::{message::Message, state::AppState, view, worker};
use iced::{Subscription, Task, widget::canvas::Cache};
use nusb::MaybeFuture;
use rfd::FileDialog;
use rindrive_core::engine::EngineType;
use rindrive_i18n::fl;
use std::{
    path::PathBuf,
    sync::{Arc, atomic::AtomicBool},
};

const DEFAULT_SECTIONS: usize = 600;
const MAX_SECTIONS: usize = 2000;
const FULLSCAN_BLOCKS: usize = 100;
const DEFAULT_BUFFER_SIZE: usize = 4096;
const MIN_BUFFER_SIZE: usize = 512;
const MAX_BUFFER_SIZE: usize = 268_435_456;

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
        match message {
            Message::SectionsChanged(val) => {
                if val.chars().all(|c| c.is_numeric()) {
                    if val.is_empty() {
                        self.sections_input = val;
                        self.block_map.clear();
                        self.map_cache.clear();
                    } else if let Ok(n) = val.parse::<usize>()
                        && n <= MAX_SECTIONS
                    {
                        self.sections_input = val;
                        self.rebuild_block_map();
                    }
                }
                Task::none()
            }

            Message::BufferSizeChanged(val) => {
                if val.chars().all(|c| c.is_numeric()) {
                    if val.is_empty() {
                        self.buffer_size_input = val;
                    } else if let Ok(n) = val.parse::<usize>()
                        && n <= MAX_BUFFER_SIZE
                    {
                        self.buffer_size_input = val;
                    }
                }
                Task::none()
            }

            Message::EngineSelected(engine) => {
                if self.is_idle() {
                    self.selected_engine = engine;
                    self.rebuild_block_map();
                }
                Task::none()
            }

            Message::SelectManual => {
                Task::perform(async { FileDialog::new().pick_folder() }, |opt| {
                    opt.map(Message::DriveDetected)
                        .unwrap_or_else(|| Message::Progress(0.0, fl!("log-cancelled")))
                })
            }

            Message::DriveDetected(path) => {
                self.log = fl!("log-detecting");
                Task::perform(fetch_drive_info(path), Message::DriveInfoLoaded)
            }

            Message::DriveInfoLoaded(info) => {
                self.selected_drive = Some(info.actual_path.clone());
                self.log = fl!(
                    "log-detected",
                    path = info.actual_path.display().to_string()
                );
                self.drive_capacity = info.drive_capacity;
                self.drive_name = info.drive_name;
                self.usb_info = info.usb_info;

                self.state = AppState::Ready;
                self.progress = 0.0;
                self.rebuild_block_map();

                Task::none()
            }

            Message::UnselectDrive => {
                if self.is_idle() {
                    self.selected_drive = None;
                    self.usb_info = None;
                    self.reset_audit_state(fl!("log-waiting").to_string());
                }
                Task::none()
            }

            Message::StartAudit => {
                self.validate_buffer_size();

                if let Some(path) = self.selected_drive.clone() {
                    self.state = AppState::Auditing;
                    self.log = fl!("log-initializing").to_string();
                    self.progress = 0.0;
                    self.block_map.fill(0);
                    self.map_cache.clear();

                    let flag = Arc::new(AtomicBool::new(false));
                    self.cancel_flag = Some(flag.clone());

                    let buffer = self
                        .buffer_size_input
                        .parse::<usize>()
                        .unwrap_or(DEFAULT_BUFFER_SIZE);

                    match self.selected_engine {
                        EngineType::SpotCheck => {
                            let sections = self
                                .sections_input
                                .parse::<usize>()
                                .unwrap_or(DEFAULT_SECTIONS);
                            Task::run(
                                worker::spotcheck::run(path, sections, buffer, flag),
                                |evt| evt,
                            )
                        }
                        EngineType::FullScan => {
                            self.block_map = vec![0; FULLSCAN_BLOCKS];
                            Task::run(worker::fullscan::run(path, buffer, flag), |evt| evt)
                        }
                    }
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

                if self.selected_engine == EngineType::FullScan {
                    let block_idx = (pct * 100.0) as usize;
                    if block_idx < self.block_map.len() {
                        self.block_map[block_idx] = 1;
                        self.map_cache.clear();
                    }
                }
                Task::none()
            }

            Message::BlockUpdated(idx, status) => {
                if let Some(block) = self.block_map.get_mut(idx)
                    && *block != status
                {
                    *block = status;
                    self.map_cache.clear();
                }
                Task::none()
            }

            Message::Finished(result) => {
                if self.state == AppState::Cancelling {
                    self.reset_audit_state(fl!("log-cancelled").to_string());
                    return Task::none();
                }

                match result {
                    Ok(report) => {
                        self.state = AppState::Finished;
                        self.progress = 100.0;
                        self.audit_time =
                            Some(chrono::Local::now().format("%m/%d/%Y %H:%M:%S").to_string());

                        self.log = if report.has_errors {
                            fl!("log-fake").to_string()
                        } else {
                            fl!("log-genuine").to_string()
                        };

                        self.block_map = report
                            .integrity_map
                            .iter()
                            .map(|&err| if err { 1 } else { 2 })
                            .collect();

                        self.map_cache.clear();
                        self.last_report = Some(report);
                    }
                    Err(e) => {
                        let err_msg = fl!("log-error", error = e.to_string());
                        self.reset_audit_state(err_msg);
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

    fn is_idle(&self) -> bool {
        !matches!(self.state, AppState::Auditing | AppState::Cancelling)
    }

    fn rebuild_block_map(&mut self) {
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

    fn reset_audit_state(&mut self, log_message: String) {
        self.state = AppState::Waiting;
        self.progress = 0.0;
        self.log = log_message;
        self.last_report = None;
        self.audit_time = None;
        self.cancel_flag = None;
        self.rebuild_block_map();
    }

    fn validate_buffer_size(&mut self) {
        if let Ok(buf_size) = self.buffer_size_input.parse::<usize>() {
            if buf_size < MIN_BUFFER_SIZE {
                self.buffer_size_input = MIN_BUFFER_SIZE.to_string();
            }
        } else {
            self.buffer_size_input = "1048576".to_string(); // 1MB fallback
        }
    }
}

async fn fetch_drive_info(path: PathBuf) -> DriveInfoResult {
    // 1. Sysinfo resolution
    let disks = sysinfo::Disks::new_with_refreshed_list();
    let mut best_disk = None;
    let mut max_len = 0;

    for disk in disks.iter() {
        let disk_name = disk.name().to_string_lossy();
        let path_str = path.to_string_lossy();

        if disk_name.starts_with(path_str.as_ref()) && path_str.starts_with("/dev/") {
            best_disk = Some(disk);
            break;
        }

        if path.starts_with(disk.mount_point()) {
            let len = disk.mount_point().as_os_str().len();
            if len > max_len {
                max_len = len;
                best_disk = Some(disk);
            }
        }
    }

    let actual_path = if let Some(disk) = &best_disk {
        let p_str = path.to_string_lossy();
        if p_str.starts_with("/dev/") || p_str.starts_with("\\\\.\\") {
            path.clone()
        } else {
            #[cfg(target_os = "linux")]
            {
                let dev_name = disk.name().to_string_lossy();
                PathBuf::from(dev_name.trim_end_matches(char::is_numeric))
            }
            #[cfg(target_os = "windows")]
            {
                disk.mount_point().to_path_buf()
            }
            #[cfg(not(any(target_os = "linux", target_os = "windows")))]
            {
                path.clone()
            }
        }
    } else {
        path.clone()
    };

    let capacity = match rindrive_core::platforms::open_drive(&actual_path.to_string_lossy()) {
        Ok(drive) => format!("{:.2} GB", drive.size() as f64 / 1_000_000_000.0),
        Err(_) => best_disk.map_or("-- GB".to_string(), |disk| {
            format!("{:.1} GB", disk.total_space() as f64 / 1_000_000_000.0)
        }),
    };

    // 2. NUSB resolution
    let mut hw_name = String::new();
    let mut hw_info = None;

    if let Ok(devices_iter) = nusb::list_devices().wait() {
        let devices: Vec<_> = devices_iter.collect();
        for dev in devices.into_iter().rev() {
            let mut is_mass_storage = dev.class() == 8;
            if !is_mass_storage {
                for interface in dev.interfaces() {
                    if interface.class() == 8 {
                        is_mass_storage = true;
                        break;
                    }
                }
            }
            if is_mass_storage {
                let mfg = dev.manufacturer_string().map(|s| s.trim().to_string());
                let prod = dev.product_string().map(|s| s.trim().to_string());

                let full_name = format!(
                    "{} {}",
                    mfg.as_deref().unwrap_or(""),
                    prod.as_deref().unwrap_or("")
                )
                .trim()
                .to_string();

                if full_name.len() > 2 {
                    hw_name = full_name;
                    hw_info = Some(UsbHardwareInfo {
                        manufacturer: mfg,
                        product: prod,
                        speed: dev.speed(),
                    });
                    break;
                }
            }
        }
    }

    DriveInfoResult {
        actual_path,
        drive_name: if hw_name.is_empty() {
            fl!("unknown-drive").to_string()
        } else {
            hw_name
        },
        drive_capacity: capacity,
        usb_info: hw_info,
    }
}
