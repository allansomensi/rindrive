use crate::message::Message;
use iced::futures::{self, SinkExt, channel::mpsc::Sender};
use rindrive_core::detector::{self, DriveEvent};
use std::path::PathBuf;
use tokio::sync::mpsc;

/// Creates an asynchronous stream that listens for hardware drive connection events.
///
/// This function acts as a bridge between the synchronous, blocking OS-level hardware
/// watcher and the asynchronous Tokio ecosystem. It yields UI messages whenever
/// a new drive is detected.
pub fn stream() -> impl futures::Stream<Item = Message> {
    iced::stream::channel(100, |mut output: Sender<Message>| async move {
        let (tokio_tx, mut tokio_rx) = mpsc::unbounded_channel::<DriveEvent>();

        std::thread::spawn(move || {
            let (std_tx, std_rx) = std::sync::mpsc::channel();

            std::thread::spawn(move || {
                let mut watcher = detector::DriveWatcher::new();
                watcher.watch(std_tx);
            });

            while let Ok(event) = std_rx.recv() {
                if tokio_tx.send(event).is_err() {
                    break;
                }
            }
        });

        while let Some(DriveEvent::Connected(path_str)) = tokio_rx.recv().await {
            let _ = output
                .send(Message::DriveDetected(PathBuf::from(path_str)))
                .await;
        }
    })
}
