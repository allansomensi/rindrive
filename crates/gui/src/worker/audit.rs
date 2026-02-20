use crate::message::Message;
use iced::futures;
use rindrive_core::{engine, platforms};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::mpsc;

pub fn run(path: PathBuf) -> impl futures::Stream<Item = Message> {
    let (tx, rx) = mpsc::channel(100);

    tokio::task::spawn_blocking(move || {
        let path_str = path.to_str().unwrap_or_default();
        match platforms::open_drive(path_str) {
            Ok(mut drive) => {
                let res = engine::spotcheck::run(&mut *drive, 10, 1024 * 1024, |msg, pct| {
                    let _ = tx.blocking_send(Message::Progress(pct, msg.to_string()));
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
