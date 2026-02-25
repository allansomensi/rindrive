use crate::{message::Message, state::AppState, view, worker};
use iced::{Subscription, Task};
use nusb::MaybeFuture;
use rfd::FileDialog;
use rindrive_core::engine::EngineType;
use rindrive_i18n::fl;
use std::{
    path::PathBuf,
    sync::{Arc, atomic::AtomicBool},
};

#[derive(Debug, Clone)]
pub struct UsbHardwareInfo {
    pub manufacturer: Option<String>,
    pub product: Option<String>,
    pub speed: Option<nusb::Speed>,
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

            // Defaults
            sections_input: "600".to_string(),
            buffer_size_input: "4096".to_string(),
            selected_engine: EngineType::SpotCheck,
            block_map: vec![0; 600],
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

            Message::UnselectDrive => {
                if !matches!(self.state, AppState::Auditing | AppState::Cancelling) {
                    self.selected_drive = None;
                    self.state = AppState::Waiting;
                    self.log = fl!("log-waiting").to_string();
                    self.progress = 0.0;
                    self.block_map.fill(0);
                    self.usb_info = None;
                    self.usb_info = None;
                    self.last_report = None;
                    self.audit_time = None;
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

                self.selected_drive = Some(actual_path.clone());
                self.log = fl!("log-detected", path = actual_path.display().to_string());
                self.state = AppState::Ready;
                self.progress = 0.0;

                let ap_str = actual_path.to_string_lossy().to_string();
                if let Ok(drive) = rindrive_core::platforms::open_drive(&ap_str) {
                    let gb = drive.size() as f64 / 1_000_000_000.0;
                    self.drive_capacity = format!("{gb:.2} GB");
                } else if let Some(disk) = best_disk {
                    let gb = disk.total_space() as f64 / 1_000_000_000.0;
                    self.drive_capacity = format!("{gb:.1} GB");
                } else {
                    self.drive_capacity = "-- GB".to_string();
                }

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

                            if !full_name.is_empty() && full_name.len() > 2 {
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

                self.usb_info = hw_info;
                self.drive_name = if hw_name.is_empty() {
                    fl!("unknown-drive").to_string()
                } else {
                    hw_name
                };

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

                match result {
                    Ok(report) => {
                        self.state = AppState::Finished;
                        self.progress = 100.0;
                        self.audit_time =
                            Some(chrono::Local::now().format("%m/%d/%Y %H:%M:%S").to_string());
                        self.last_report = Some(report.clone());
                        self.log = if report.has_errors {
                            fl!("log-fake").to_string()
                        } else {
                            fl!("log-genuine").to_string()
                        };
                    }
                    Err(e) => {
                        self.state = AppState::Ready;
                        self.log = fl!("log-error", error = e.to_string());
                        self.progress = 0.0;

                        if let Ok(n) = self.sections_input.parse::<usize>() {
                            self.block_map = vec![0; n];
                        }
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
