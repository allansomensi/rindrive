use std::sync::mpsc::Sender;
use std::{thread, time::Duration};

/// Enum representing events related to storage drives.
pub enum DriveEvent {
    /// On Windows, contains the drive letter (e.g., "E:\").
    /// On Linux, contains the physical device (e.g., "/dev/sdb").
    Connected(String),
}

/// Monitors the system for new storage devices using a polling mechanism.
///
/// This struct leverages [`sysinfo`] to be cross-platform (Windows, Linux, macOS).
/// It detects new mount points by comparing the current list of disks against a known history.
pub struct DriveWatcher {
    known_disks: Vec<String>,
}

impl Default for DriveWatcher {
    fn default() -> Self {
        Self::new()
    }
}

impl DriveWatcher {
    /// Creates a new [`DriveWatcher`] and captures the initial state of connected disks.
    ///
    /// Any disk currently connected will be added to the internal ignore list
    /// so that events are only fired for *new* connections.
    pub fn new() -> Self {
        Self {
            known_disks: Self::get_connected_drives(),
        }
    }

    /// Starts the monitoring loop in the current thread.
    ///
    /// This method blocks indefinitely. It polls the system every 500ms to check
    /// for changes in the mounted disks list.
    pub fn watch(&mut self, tx: Sender<DriveEvent>) {
        loop {
            thread::sleep(Duration::from_millis(500));

            let current_drives = Self::get_connected_drives();

            for drive in current_drives {
                if !self.known_disks.contains(&drive) {
                    self.known_disks.push(drive.clone());

                    if tx.send(DriveEvent::Connected(drive)).is_err() {
                        return;
                    }
                }
            }
        }
    }

    /// Windows implementation
    #[cfg(target_os = "windows")]
    fn get_connected_drives() -> Vec<String> {
        use sysinfo::Disks;
        Disks::new_with_refreshed_list()
            .iter()
            .map(|d| d.mount_point().to_string_lossy().to_string())
            .collect()
    }

    /// Linux implementation
    #[cfg(target_os = "linux")]
    fn get_connected_drives() -> Vec<String> {
        let mut drives = Vec::new();

        if let Ok(entries) = std::fs::read_dir("/sys/class/block") {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();

                if name.starts_with("sd") && name.len() == 3 {
                    let removable_path = format!("/sys/class/block/{name}/removable");
                    if let Ok(removable_flag) = std::fs::read_to_string(removable_path)
                        && removable_flag.trim() == "1"
                    {
                        drives.push(format!("/dev/{name}"));
                    }
                }
            }
        }
        drives
    }
}
