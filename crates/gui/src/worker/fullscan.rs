use crate::message::Message;
use iced::futures;
use rindrive_core::{engine, platforms};
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;

pub fn run(
    path: PathBuf,
    buffer: usize,
    cancel_flag: Arc<AtomicBool>,
) -> impl futures::Stream<Item = Message> {
    let (tx, rx) = mpsc::unbounded_channel();

    tokio::task::spawn_blocking(move || {
        let path_str = path.to_str().unwrap_or_default();
        match platforms::open_drive(path_str) {
            Ok(mut drive) => {
                let res = engine::fullscan::run(&mut *drive, buffer, |msg, pct, _| {
                    let _ = tx.send(Message::Progress(pct, msg));

                    let (phase_pct, status) = if pct < 0.5 {
                        (pct * 2.0, 4u8)
                    } else {
                        ((pct - 0.5) * 2.0, 3u8)
                    };
                    let block_idx = (phase_pct * 100.0) as usize;
                    if block_idx < 100 {
                        let _ = tx.send(Message::BlockUpdated(block_idx, status));
                    }

                    !cancel_flag.load(Ordering::Relaxed)
                });
                let _ = tx.send(Message::Finished(
                    res.map(Arc::new).map_err(|e| e.to_string()),
                ));
            }
            Err(e) => {
                let _ = tx.send(Message::Finished(Err(e.to_string())));
            }
        }
    });

    UnboundedReceiverStream::new(rx)
}
