#[cfg(target_os = "linux")]
use super::PhysicalDrive;
#[cfg(target_os = "linux")]
use std::{
    fs::{File, OpenOptions},
    io::{self, BufRead, BufReader, Seek, SeekFrom},
    os::unix::fs::{FileExt, OpenOptionsExt},
};

/// Represents a physical drive on Linux.
#[cfg(target_os = "linux")]
pub struct LinuxPhysicalDrive {
    file: File,
    path: String,
    size: u64,
}

#[cfg(target_os = "linux")]
unsafe impl Send for LinuxPhysicalDrive {}

#[cfg(target_os = "linux")]
impl LinuxPhysicalDrive {
    pub fn open(path_or_mount: &str) -> Result<Self, String> {
        let physical_path = if path_or_mount.starts_with("/dev/") {
            path_or_mount.to_string()
        } else {
            Self::get_physical_path(path_or_mount)?
        };

        let mut options = OpenOptions::new();
        options.read(true).write(true);

        // O_DIRECT = Raw hardware access (bypassing OS cache)
        // O_SYNC = Synchronous writes
        // O_EXCL = Exclusive access
        let flags = libc::O_DIRECT | libc::O_SYNC | libc::O_EXCL;
        options.custom_flags(flags);

        let mut file = options.open(&physical_path).map_err(|e| {
            if e.raw_os_error() == Some(libc::EBUSY) {
                fl!("linux-err-drive-busy", path = physical_path.clone())
            } else {
                fl!(
                    "linux-err-open-device",
                    path = physical_path.clone(),
                    error = e.to_string()
                )
            }
        })?;

        let size = file.seek(SeekFrom::End(0)).map_err(|e| e.to_string())?;

        Ok(Self {
            file,
            path: physical_path,
            size,
        })
    }

    fn get_physical_path(mount_point: &str) -> Result<String, String> {
        let mounts_file = File::open("/proc/mounts")
            .map_err(|e| fl!("linux-err-read-mounts", error = e.to_string()))?;
        let reader = BufReader::new(mounts_file);

        let target_mount = mount_point.trim_end_matches('/');

        for line in reader.lines() {
            let line = line.unwrap_or_default();
            let parts: Vec<&str> = line.split_whitespace().collect();

            if parts.len() >= 2 {
                let current_mount = parts[1].trim_end_matches('/');
                if current_mount == target_mount || parts[1] == mount_point {
                    return Ok(parts[0].to_string());
                }
            }
        }

        Err(fl!("linux-err-mount-not-found", path = mount_point))
    }
}

#[cfg(target_os = "linux")]
impl PhysicalDrive for LinuxPhysicalDrive {
    fn path(&self) -> &str {
        &self.path
    }

    fn size(&self) -> u64 {
        self.size
    }

    fn read_at(&mut self, offset: u64, buffer: &mut [u8]) -> io::Result<()> {
        self.file.read_exact_at(buffer, offset)?;
        Ok(())
    }

    fn write_at(&mut self, offset: u64, data: &[u8]) -> io::Result<()> {
        self.file.write_all_at(data, offset)?;
        Ok(())
    }

    fn sync(&mut self) -> io::Result<()> {
        self.file.sync_all()?;
        Ok(())
    }
}
