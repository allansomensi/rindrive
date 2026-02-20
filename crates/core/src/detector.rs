use std::sync::mpsc::Sender;
use std::{thread, time::Duration};
use sysinfo::Disks;

/// Enum representing events related to storage drives.
pub enum DriveEvent {
    /// Triggered when a new mount point is detected. Contains the path (e.g., "E:\" or "/mnt/usb").
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
        let disks = Disks::new_with_refreshed_list();
        let known_disks = disks
            .iter()
            .map(|d| d.mount_point().to_string_lossy().to_string())
            .collect();
        Self { known_disks }
    }

    /// Starts the monitoring loop in the current thread.
    ///
    /// This method blocks indefinitely. It polls the system every 500ms to check
    /// for changes in the mounted disks list.
    ///
    /// # Arguments
    /// * `tx` - A generic channel sender to transmit [`DriveEvent`].
    ///
    /// # Behavior
    /// * If the channel is disconnected (receiver dropped), the loop terminates.
    /// * It currently only detects **new** connections (append-only logic).
    pub fn watch(&mut self, tx: Sender<DriveEvent>) {
        // We maintain a local Disks object to refresh the system state
        let mut disks = Disks::new();
        loop {
            thread::sleep(Duration::from_millis(500));

            // true = retrieve full info (space, type, etc), though we only need the list.
            disks.refresh(true);

            for disk in &disks {
                let mp = disk.mount_point().to_string_lossy().to_string();

                // If we haven't seen this mount point before, it's a new connection
                if !self.known_disks.contains(&mp) {
                    self.known_disks.push(mp.clone());

                    if tx.send(DriveEvent::Connected(mp)).is_err() {
                        // Receiver dropped, stop watching to prevent zombie threads
                        return;
                    }
                }
            }
        }
    }
}
