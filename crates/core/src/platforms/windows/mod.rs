#[cfg(target_os = "windows")]
use super::PhysicalDrive;
#[cfg(target_os = "windows")]
use rindrive_i18n::fl;
#[cfg(target_os = "windows")]
use std::io;
#[cfg(target_os = "windows")]
use windows::{
    Win32::{
        Foundation::{CloseHandle, GENERIC_READ, GENERIC_WRITE, HANDLE},
        Storage::FileSystem::{
            CreateFileW, FILE_BEGIN, FILE_FLAG_NO_BUFFERING, FILE_FLAG_WRITE_THROUGH,
            FILE_SHARE_READ, FILE_SHARE_WRITE, FlushFileBuffers, OPEN_EXISTING, ReadFile,
            SetFilePointerEx, WriteFile,
        },
        System::{
            IO::DeviceIoControl,
            Ioctl::{
                DISK_GEOMETRY_EX, FSCTL_ALLOW_EXTENDED_DASD_IO, FSCTL_DISMOUNT_VOLUME,
                FSCTL_LOCK_VOLUME, IOCTL_DISK_GET_DRIVE_GEOMETRY_EX,
                IOCTL_STORAGE_GET_DEVICE_NUMBER, STORAGE_DEVICE_NUMBER,
            },
        },
    },
    core::PCWSTR,
};

/// Represents a physical drive on Windows with raw access capabilities.
///
/// This struct manages the lifecycle of the drive handle and an optional
/// volume lock handle to ensure exclusive access during operations.
#[cfg(target_os = "windows")]
pub struct WindowsPhysicalDrive {
    handle: HANDLE,
    volume_lock_handle: Option<HANDLE>,
    path: String,
    size: u64,
}

#[cfg(target_os = "windows")]
unsafe impl Send for WindowsPhysicalDrive {}

#[cfg(target_os = "windows")]
impl WindowsPhysicalDrive {
    /// Opens a physical drive associated with the given mount point (e.g., "C:").
    ///
    /// This method performs the following:
    /// 1. Resolves the physical drive path (e.g., `\\.\PhysicalDrive0`).
    /// 2. Acquires a lock on the logical volume to prevent OS interference.
    /// 3. Opens the physical drive with `NO_BUFFERING` and `WRITE_THROUGH` flags.
    ///
    /// # Errors
    /// Returns an error if the process lacks Administrator privileges or if the
    /// volume cannot be locked.
    pub fn open(mount_point: &str) -> Result<Self, String> {
        let physical_path = Self::get_physical_path(mount_point)?;

        unsafe {
            // Acquire exclusive lock on the volume
            let volume_handle = Self::get_volume_lock_handle(mount_point)?;

            let path_wide: Vec<u16> = physical_path
                .encode_utf16()
                .chain(std::iter::once(0))
                .collect();

            let handle = CreateFileW(
                PCWSTR(path_wide.as_ptr()),
                (GENERIC_READ | GENERIC_WRITE).0,
                Default::default(), // No sharing allowed
                None,
                OPEN_EXISTING,
                FILE_FLAG_NO_BUFFERING | FILE_FLAG_WRITE_THROUGH,
                None,
            )
            .map_err(|e| fl!("win-err-open-drive", error = e.to_string()))?;

            // Enable Extended DASD I/O to allow access beyond the filesystem boundary
            let mut b_ret = 0;
            let _ = DeviceIoControl(
                handle,
                FSCTL_ALLOW_EXTENDED_DASD_IO,
                None,
                0,
                None,
                0,
                Some(&mut b_ret),
                None,
            );

            let size = Self::get_total_bytes(handle).map_err(|e| e.to_string())?;

            Ok(Self {
                handle,
                volume_lock_handle: Some(volume_handle),
                path: physical_path,
                size,
            })
        }
    }

    /// Retrieves the total size of the disk in bytes using [`IOCTL_DISK_GET_DRIVE_GEOMETRY_EX`].
    fn get_total_bytes(handle: HANDLE) -> io::Result<u64> {
        let mut geometry = DISK_GEOMETRY_EX::default();
        let mut bytes_returned = 0u32;
        unsafe {
            DeviceIoControl(
                handle,
                IOCTL_DISK_GET_DRIVE_GEOMETRY_EX,
                None,
                0,
                Some(&mut geometry as *mut _ as *mut _),
                std::mem::size_of::<DISK_GEOMETRY_EX>() as u32,
                Some(&mut bytes_returned),
                None,
            )
            .map_err(|_| io::Error::last_os_error())?;
            Ok(geometry.DiskSize as u64)
        }
    }

    /// Opens a handle to the logical volume, dismounts it, and applies a filesystem lock.
    ///
    /// This ensures that no other process or the OS itself writes to the volume
    /// while the physical drive is being manipulated.
    fn get_volume_lock_handle(mount_point: &str) -> Result<HANDLE, String> {
        let volume = mount_point.trim_end_matches('\\');
        let raw_path = format!("\\\\.\\{volume}");
        let path_wide: Vec<u16> = raw_path.encode_utf16().chain(std::iter::once(0)).collect();

        unsafe {
            let h_volume = CreateFileW(
                PCWSTR(path_wide.as_ptr()),
                (GENERIC_READ | GENERIC_WRITE).0,
                FILE_SHARE_READ | FILE_SHARE_WRITE,
                None,
                OPEN_EXISTING,
                Default::default(),
                None,
            )
            .map_err(|e| fl!("win-err-lock-failure", error = e.to_string()))?;

            let mut b_ret = 0;
            // Force the filesystem to dismount and lock
            let _ = DeviceIoControl(
                h_volume,
                FSCTL_DISMOUNT_VOLUME,
                None,
                0,
                None,
                0,
                Some(&mut b_ret),
                None,
            );
            let _ = DeviceIoControl(
                h_volume,
                FSCTL_LOCK_VOLUME,
                None,
                0,
                None,
                0,
                Some(&mut b_ret),
                None,
            );
            Ok(h_volume)
        }
    }

    /// Resolves a mount point (e.g., "C:") to its underlying physical drive path (e.g., "\\.\PhysicalDrive0").
    fn get_physical_path(mount_point: &str) -> Result<String, String> {
        let volume = mount_point.trim_end_matches('\\');
        let raw_path = format!("\\\\.\\{volume}");
        let path_wide: Vec<u16> = raw_path.encode_utf16().chain(std::iter::once(0)).collect();

        unsafe {
            let h_volume = CreateFileW(
                PCWSTR(path_wide.as_ptr()),
                GENERIC_READ.0,
                FILE_SHARE_READ | FILE_SHARE_WRITE,
                None,
                OPEN_EXISTING,
                Default::default(),
                None,
            )
            .map_err(|e| fl!("win-err-map-error", error = e.to_string()))?;

            let mut dev_number = STORAGE_DEVICE_NUMBER::default();
            let mut bytes_ret = 0;
            let res = DeviceIoControl(
                h_volume,
                IOCTL_STORAGE_GET_DEVICE_NUMBER,
                None,
                0,
                Some(&mut dev_number as *mut _ as *mut _),
                std::mem::size_of::<STORAGE_DEVICE_NUMBER>() as u32,
                Some(&mut bytes_ret),
                None,
            );
            let _ = CloseHandle(h_volume);

            if res.is_ok() {
                Ok(format!("\\\\.\\PhysicalDrive{}", dev_number.DeviceNumber))
            } else {
                Err(fl!("win-err-disk-number"))
            }
        }
    }
}

#[cfg(target_os = "windows")]
impl PhysicalDrive for WindowsPhysicalDrive {
    fn path(&self) -> &str {
        &self.path
    }

    fn size(&self) -> u64 {
        self.size
    }

    /// Reads data from the device at the specified offset.
    ///
    /// **Note:** Due to [`FILE_FLAG_NO_BUFFERING`], the buffer pointer and size,
    /// as well as the offset, must usually be sector-aligned.
    fn read_at(&mut self, offset: u64, buffer: &mut [u8]) -> io::Result<()> {
        unsafe {
            SetFilePointerEx(self.handle, offset as i64, None, FILE_BEGIN)
                .map_err(|_| io::Error::last_os_error())?;

            let mut bytes_read = 0;

            ReadFile(self.handle, Some(buffer), Some(&mut bytes_read), None)
                .map_err(|_| io::Error::last_os_error())?;

            if bytes_read == 0 && !buffer.is_empty() {
                return Err(io::Error::new(
                    io::ErrorKind::UnexpectedEof,
                    fl!("win-err-read-zero"),
                ));
            }
        }
        Ok(())
    }

    /// Writes data to the device at the specified offset.
    ///
    /// **Note:** Requires strict sector alignment for the buffer and offset due to
    /// unbuffered I/O settings.
    fn write_at(&mut self, offset: u64, data: &[u8]) -> io::Result<()> {
        unsafe {
            SetFilePointerEx(self.handle, offset as i64, None, FILE_BEGIN)
                .map_err(|_| io::Error::last_os_error())?;

            let mut bytes_written = 0;

            WriteFile(self.handle, Some(data), Some(&mut bytes_written), None)
                .map_err(|_| io::Error::last_os_error())?;
        }
        Ok(())
    }

    /// Flushes the metadata and buffers of the device handle.
    fn sync(&mut self) -> io::Result<()> {
        unsafe {
            FlushFileBuffers(self.handle).map_err(|_| io::Error::last_os_error())?;
        }
        Ok(())
    }
}

#[cfg(target_os = "windows")]
impl Drop for WindowsPhysicalDrive {
    /// Closes the physical drive handle and the volume lock handle, releasing resources.
    fn drop(&mut self) {
        unsafe {
            if !self.handle.is_invalid() {
                let _ = CloseHandle(self.handle);
            }
            if let Some(h) = self.volume_lock_handle {
                let _ = CloseHandle(h);
            }
        }
    }
}
