use crate::platforms::PhysicalDrive;
use rand::{RngExt, seq::SliceRandom};
use std::io;

#[derive(Debug, Clone)]
pub struct Report {
    pub declared_size_bytes: u64,
    pub validated_size_bytes: u64,
    pub has_errors: bool,
    pub integrity_map: Vec<bool>,
}

pub fn run<F>(
    drive: &mut dyn PhysicalDrive,
    sections: usize,
    buffer_size: usize,
    progress_callback: F,
) -> io::Result<Report>
where
    F: Fn(String, f32),
{
    let total_bytes = drive.size();

    // Safety check to avoid division by zero
    if sections == 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Sections must be greater than 0",
        ));
    }

    // Calculations based on user input
    let total_sectors = total_bytes / 512;
    let area_size = total_sectors / sections as u64;

    // Calculate how many sectors our buffer occupies
    let sectors_per_buffer = (buffer_size as u64 + 511).div_ceil(512);

    // Calculate test points (offsets) across the drive
    let test_points: Vec<u64> = (0..sections)
        .map(|i| {
            let base_sector = i as u64 * area_size;
            // Aim for the end of the area, minus the buffer size
            (base_sector + area_size).saturating_sub(sectors_per_buffer)
        })
        .collect();

    // 1. Backup Phase
    progress_callback("Backing up data...".into(), 0.0);
    let mut backups = Vec::with_capacity(sections);
    for (i, &sector) in test_points.iter().enumerate() {
        let mut buf = vec![0u8; buffer_size];
        drive.read_at(sector * 512, &mut buf)?;
        backups.push(buf);

        if i % 10 == 0 || i == sections - 1 {
            progress_callback(
                format!("Backup block {}/{}", i + 1, sections),
                0.1 * (i as f32 / sections as f32),
            );
        }
    }

    // 2. Poisoning Phase
    progress_callback("Poisoning drive regions...".into(), 0.1);
    let mut poisons = Vec::with_capacity(sections);
    let mut write_order: Vec<usize> = (0..sections).collect();
    let mut rng = rand::rng();
    write_order.shuffle(&mut rng);

    for (count, &idx) in write_order.iter().enumerate() {
        let mut p_buf = vec![0u8; buffer_size];
        rng.fill(&mut p_buf[..]);

        drive.write_at(test_points[idx] * 512, &p_buf)?;
        poisons.push((idx, p_buf));

        let progress = 0.1 + (0.4 * (count as f32 / sections as f32));
        if count % 10 == 0 || count == sections - 1 {
            progress_callback("Writing random data...".into(), progress);
        }
    }

    // Sort poisons by index so they match the order of test_points for verification
    poisons.sort_by_key(|k| k.0);
    drive.sync()?;

    // 3. Verification Phase
    progress_callback("Verifying integrity...".into(), 0.5);
    let mut results = vec![false; sections];

    for i in 0..sections {
        let mut check_buf = vec![0u8; buffer_size];

        match drive.read_at(test_points[i] * 512, &mut check_buf) {
            Ok(_) => {
                let expected = &poisons[i].1;
                if &check_buf == expected {
                    results[i] = true;
                }
            }
            Err(_) => results[i] = false,
        }
        let progress = 0.5 + (0.4 * (i as f32 / sections as f32));
        if i % 10 == 0 || i == sections - 1 {
            progress_callback("Verifying...".into(), progress);
        }
    }

    // 4. Restore Phase
    progress_callback("Restoring original data...".into(), 0.9);
    for i in 0..sections {
        let _ = drive.write_at(test_points[i] * 512, &backups[i]);
    }
    drive.sync()?;

    // Result Analysis
    let first_fail = results.iter().position(|&r| !r).unwrap_or(sections);
    let valid_ratio = first_fail as f64 / sections as f64;
    let valid_bytes = (valid_ratio * total_bytes as f64) as u64;

    Ok(Report {
        declared_size_bytes: total_bytes,
        validated_size_bytes: valid_bytes,
        has_errors: first_fail < sections,
        integrity_map: results,
    })
}
