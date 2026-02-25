use crate::message::Message;
use iced::futures;
use rindrive_core::{engine, platforms};
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;

/// Spawns a background worker to execute the spotcheck drive audit.
///
/// This function acts as an asynchronous bridge between the synchronous
/// I/O operations of the core engine and the reactive Iced UI. It runs the
/// engine in a dedicated blocking thread and streams UI updates back via a channel.
pub fn run(
    path: PathBuf,
    sections: usize,
    buffer: usize,
    cancel_flag: Arc<AtomicBool>,
) -> impl futures::Stream<Item = Message> {
    let (tx, rx) = mpsc::unbounded_channel();

    tokio::task::spawn_blocking(move || {
        let path_str = path.to_str().unwrap_or_default();
        match platforms::open_drive(path_str) {
            Ok(mut drive) => {
                let res = engine::spotcheck::run(
                    &mut *drive,
                    sections,
                    buffer,
                    |msg, pct, block_update| {
                        if let Some((idx, state)) = block_update {
                            let _ = tx.send(Message::BlockUpdated(idx, state));
                        }
                        let _ = tx.send(Message::Progress(pct, msg.to_string()));
                        !cancel_flag.load(Ordering::Relaxed)
                    },
                );
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
