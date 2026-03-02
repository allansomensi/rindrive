use crate::app::DriveInfoResult;
use rindrive_core::engine::{EngineType, spotcheck::Report};
use std::{path::PathBuf, sync::Arc};

#[derive(Debug, Clone)]
pub enum Message {
    SelectManual,
    DriveDetected(PathBuf),
    DriveInfoLoaded(DriveInfoResult),
    UnselectDrive,
    StartAudit,
    CancelAudit,
    Progress(f32, String),
    BlockUpdated(usize, u8),
    Finished(Result<Arc<Report>, String>),
    SectionsChanged(String),
    BufferSizeChanged(String),
    EngineSelected(EngineType),
}
