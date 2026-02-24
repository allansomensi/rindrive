use rindrive_core::engine::{self, EngineType};
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub enum Message {
    SelectManual,
    DriveDetected(PathBuf),
    UnselectDrive,
    StartAudit,
    CancelAudit,
    Progress(f32, String),
    BlockUpdated(usize, u8),
    Finished(Result<Arc<engine::spotcheck::Report>, String>),
    SectionsChanged(String),
    BufferSizeChanged(String),
    EngineSelected(EngineType),
}
