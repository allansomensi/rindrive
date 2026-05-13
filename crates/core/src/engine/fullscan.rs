use crate::{
    engine::{AlignedBuffer, spotcheck::Report},
    platforms::PhysicalDrive,
};
use rindrive_i18n::fl;
use std::io;

pub fn run<F>(
    drive: &mut dyn PhysicalDrive,
    buffer_size: usize,
    mut progress_callback: F,
) -> io::Result<Report>
where
    F: FnMut(String, f32, Option<(usize, u8)>) -> bool,
{
    let total_bytes = drive.size();
    let block_size = buffer_size as u64;
    let total_blocks = total_bytes / block_size;

    if total_blocks == 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            fl!("error-drive-too-small"),
        ));
    }

    let mut results = vec![false; 100];

    // 1. Write
    for i in 0..total_blocks {
        let offset = i * block_size;
        let pct = (i as f32 / total_blocks as f32) * 0.5;

        if !progress_callback(fl!("audit-writing-random"), pct, None) {
            return Err(io::Error::new(io::ErrorKind::Interrupted, "Cancelled"));
        }

        let mut buf = AlignedBuffer::new(buffer_size);
        let slice = buf.as_mut_slice();

        for (idx, byte) in slice.iter_mut().enumerate() {
            *byte = ((offset + idx as u64) % 256) as u8;
        }

        drive.write_at(offset, slice)?;
    }

    // 2. Read and verify
    let mut errors_found = false;
    let mut last_valid_block = 0;

    for i in 0..total_blocks {
        let offset = i * block_size;
        let pct = 0.5 + ((i as f32 / total_blocks as f32) * 0.5);

        if !progress_callback(fl!("audit-verifying"), pct, None) {
            return Err(io::Error::new(io::ErrorKind::Interrupted, "Cancelled"));
        }

        let mut read_buf = AlignedBuffer::new(buffer_size);
        drive.read_at(offset, read_buf.as_mut_slice())?;

        let mut is_block_valid = true;
        for (idx, &byte) in read_buf.as_slice().iter().enumerate() {
            if byte != ((offset + idx as u64) % 256) as u8 {
                is_block_valid = false;
                errors_found = true;
                break;
            }
        }

        if is_block_valid && !errors_found {
            last_valid_block = i;
        }

        let map_idx = (i * 100 / total_blocks) as usize;
        if map_idx < 100 {
            results[map_idx] = is_block_valid;
        }
    }

    let validated_size = (last_valid_block + 1) * block_size;

    Ok(Report {
        declared_size_bytes: total_bytes,
        validated_size_bytes: if errors_found {
            validated_size
        } else {
            total_bytes
        },
        has_errors: errors_found,
        integrity_map: results,
    })
}
