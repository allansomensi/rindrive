use crate::{
    app::{
        App, DEFAULT_BUFFER_SIZE, DEFAULT_SECTIONS, DriveInfoResult, FULLSCAN_BLOCKS,
        MAX_BUFFER_SIZE, MAX_SECTIONS, UsbHardwareInfo,
    },
    message::Message,
    state::AppState,
    worker,
};
use iced::Task;
use nusb::MaybeFuture;
use rfd::FileDialog;
use rindrive_core::engine::EngineType;
use rindrive_i18n::fl;
use std::{
    path::PathBuf,
    sync::{Arc, atomic::AtomicBool},
};

pub fn handle_message(app: &mut App, message: Message) -> Task<Message> {
    match message {
        Message::SectionsChanged(val) => {
            if val.chars().all(|c| c.is_numeric()) {
                if val.is_empty() {
                    app.sections_input = val;
                    app.block_map.clear();
                    app.map_cache.clear();
                } else if let Ok(n) = val.parse::<usize>()
                    && n <= MAX_SECTIONS
                {
                    app.sections_input = val;
                    app.rebuild_block_map();
                }
            }
            Task::none()
        }

        Message::BufferSizeChanged(val) => {
            if val.chars().all(|c| c.is_numeric()) {
                if val.is_empty() {
                    app.buffer_size_input = val;
                } else if let Ok(n) = val.parse::<usize>()
                    && n <= MAX_BUFFER_SIZE
                {
                    app.buffer_size_input = val;
                }
            }
            Task::none()
        }

        Message::EngineSelected(engine) => {
            if app.is_idle() {
                app.selected_engine = engine;
                app.rebuild_block_map();
            }
            Task::none()
        }

        Message::SelectManual => Task::perform(async { FileDialog::new().pick_folder() }, |opt| {
            opt.map(Message::DriveDetected)
                .unwrap_or_else(|| Message::Progress(0.0, fl!("log-cancelled")))
        }),

        Message::DriveDetected(path) => {
            app.log = fl!("log-detecting");

            Task::perform(
                async move {
                    tokio::task::spawn_blocking(move || fetch_drive_info_sync(path))
                        .await
                        .expect("Hardware fetch task panicked")
                },
                Message::DriveInfoLoaded,
            )
        }

        Message::DriveInfoLoaded(info) => {
            app.selected_drive = Some(info.actual_path.clone());
            app.log = fl!(
                "log-detected",
                path = info.actual_path.display().to_string()
            );
            app.drive_capacity = info.drive_capacity;
            app.drive_name = info.drive_name;
            app.usb_info = info.usb_info;

            app.state = AppState::Ready;
            app.progress = 0.0;
            app.rebuild_block_map();
            Task::none()
        }

        Message::UnselectDrive => {
            if app.is_idle() {
                app.selected_drive = None;
                app.usb_info = None;
                app.reset_audit_state(fl!("log-waiting").to_string());
            }
            Task::none()
        }

        Message::StartAudit => {
            app.validate_buffer_size();

            if let Some(path) = app.selected_drive.clone() {
                app.state = AppState::Auditing;
                app.log = fl!("log-initializing").to_string();
                app.progress = 0.0;
                app.block_map.fill(0);
                app.map_cache.clear();

                let flag = Arc::new(AtomicBool::new(false));
                app.cancel_flag = Some(flag.clone());

                let buffer = app
                    .buffer_size_input
                    .parse::<usize>()
                    .unwrap_or(DEFAULT_BUFFER_SIZE);

                match app.selected_engine {
                    EngineType::SpotCheck => {
                        let sections = app
                            .sections_input
                            .parse::<usize>()
                            .unwrap_or(DEFAULT_SECTIONS);
                        Task::run(
                            worker::spotcheck::run(path, sections, buffer, flag),
                            |evt| evt,
                        )
                    }
                    EngineType::FullScan => {
                        app.block_map = vec![0; FULLSCAN_BLOCKS];
                        Task::run(worker::fullscan::run(path, buffer, flag), |evt| evt)
                    }
                }
            } else {
                Task::none()
            }
        }

        Message::CancelAudit => {
            if let Some(flag) = &app.cancel_flag {
                flag.store(true, std::sync::atomic::Ordering::Relaxed);
            }
            app.state = AppState::Cancelling;
            app.log = fl!("log-cancelling").to_string();
            Task::none()
        }

        Message::Progress(pct, msg) => {
            app.progress = pct * 100.0;
            app.log = msg;
            Task::none()
        }

        Message::BlockUpdated(idx, status) => {
            if let Some(block) = app.block_map.get_mut(idx)
                && *block != status
            {
                *block = status;
                app.map_cache.clear();
            }
            Task::none()
        }

        Message::Finished(result) => {
            if app.state == AppState::Cancelling {
                app.reset_audit_state(fl!("log-cancelled").to_string());
                return Task::none();
            }

            match result {
                Ok(report) => {
                    app.state = AppState::Finished;
                    app.progress = 100.0;
                    app.audit_time =
                        Some(chrono::Local::now().format("%m/%d/%Y %H:%M:%S").to_string());

                    app.log = if report.has_errors {
                        fl!("log-fake").to_string()
                    } else {
                        fl!("log-genuine").to_string()
                    };

                    app.block_map = report
                        .integrity_map
                        .iter()
                        .map(|&ok| if ok { 1 } else { 2 })
                        .collect();

                    app.map_cache.clear();
                    app.last_report = Some(report);
                }
                Err(e) => {
                    let err_msg = fl!("log-error", error = e.to_string());
                    app.reset_audit_state(err_msg);
                }
            };
            Task::none()
        }
    }
}

fn fetch_drive_info_sync(path: PathBuf) -> DriveInfoResult {
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

    let capacity = best_disk.map_or_else(
        || "-- GB".to_string(),
        |disk| format!("{:.1} GB", disk.total_space() as f64 / 1_000_000_000.0),
    );

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
