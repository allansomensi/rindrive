use crate::message::Message;
use iced::futures;
use rindrive_core::{engine, platforms};
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::sync::mpsc;

pub fn run(
    path: PathBuf,
    sections_str: String,
    buffer_str: String,
    cancel_flag: Arc<AtomicBool>,
) -> impl futures::Stream<Item = Message> {
    let sections = sections_str.parse::<usize>().unwrap_or(10);
    let buffer = buffer_str.parse::<usize>().unwrap_or(1024 * 1024);

    let (tx, rx) = mpsc::channel(100);

    tokio::task::spawn_blocking(move || {
        let path_str = path.to_str().unwrap_or_default();
        match platforms::open_drive(path_str) {
            Ok(mut drive) => {
                let res = engine::spotcheck::run(&mut *drive, sections, buffer, |msg, pct| {
                    let _ = tx.blocking_send(Message::Progress(pct, msg.to_string()));

                    !cancel_flag.load(Ordering::Relaxed)
                });

                let _ = tx.blocking_send(Message::Finished(
                    res.map(Arc::new).map_err(|e| e.to_string()),
                ));
            }
            Err(e) => {
                let _ = tx.blocking_send(Message::Finished(Err(e.to_string())));
            }
        }
    });

    tokio_stream::wrappers::ReceiverStream::new(rx)
}
