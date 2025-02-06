use std::{path::PathBuf, time::Instant};

pub enum BackupEvent {
    Backup { name: String },
    Restore { name: String },
}

pub enum ChangeEvent {
    Create { path: PathBuf },
    Delete { path: PathBuf },
    Modify { path: PathBuf },
    Rename { from: PathBuf, to: PathBuf },
}

pub enum EngineEventKind {
    Backup(BackupEvent),
    Change(ChangeEvent),
}

pub struct EngineEvent {
    timestamp: Instant,
    kind: EngineEventKind,
}
