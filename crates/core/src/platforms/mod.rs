use std::io;

/// A trait representing a physical storage device, providing low-level read/write access.
///
/// This abstraction allows the main engine to interact with drives uniformly across
/// different operating systems (Windows, Linux, macOS).
pub trait PhysicalDrive: Send {
    /// Returns the raw path to the device (e.g., "\\.\PhysicalDrive1" or "/dev/sdb").
    fn path(&self) -> &str;

    /// Returns the total size of the drive in bytes.
    fn size(&self) -> u64;

    /// Reads bytes from the device at a specific offset.
    fn read_at(&mut self, offset: u64, buffer: &mut [u8]) -> io::Result<()>;

    /// Writes bytes to the device at a specific offset.
    ///
    /// **Note:** Depending on the platform, writes may need to be sector-aligned.
    fn write_at(&mut self, offset: u64, data: &[u8]) -> io::Result<()>;

    /// Flushes any buffered data to the physical device to ensure data integrity.
    fn sync(&mut self) -> io::Result<()>;
}

// ================= WINDOWS =================
#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "windows")]
pub use windows::WindowsPhysicalDrive as PlatformDrive;

/// Factory function to open a drive, returning a platform-specific implementation.
///
/// This function handles the logic of selecting the correct driver for the current
/// operating system and resolving the mount point to a raw physical device.
///
/// # Arguments
/// * `mount_point` - The file system path where the drive is mounted (e.g., "E:" on Windows or "/media/usb" on Linux).
pub fn open_drive(mount_point: &str) -> Result<Box<dyn PhysicalDrive>, String> {
    #[cfg(target_os = "windows")]
    {
        let drive = PlatformDrive::open(mount_point)?;
        Ok(Box::new(drive))
    }
}
