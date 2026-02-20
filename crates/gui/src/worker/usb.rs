use crate::message::Message;
use iced::futures::channel::mpsc::Sender;
use iced::futures::{self, SinkExt};
use rindrive_core::detector;
use std::path::PathBuf;
use std::time::Duration;

pub fn stream() -> impl futures::Stream<Item = Message> {
    iced::stream::channel(100, |mut output: Sender<Message>| async move {
        let (tx, rx) = std::sync::mpsc::channel();

        std::thread::spawn(move || {
            let mut watcher = detector::DriveWatcher::new();
            watcher.watch(tx);
        });

        loop {
            if let Ok(detector::DriveEvent::Connected(path_str)) = rx.try_recv() {
                let _ = output
                    .send(Message::DriveDetected(PathBuf::from(path_str)))
                    .await;
            }
            tokio::time::sleep(Duration::from_millis(500)).await;
        }
    })
}
