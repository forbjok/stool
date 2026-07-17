use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};

use tracing::{error, info};
use tracing_subscriber::{EnvFilter, FmtSubscriber};

use crate::{
    engine::{self, EngineArgs, EngineState, ui::StoolUiHandler},
    internal::sync::SyncUiHandler,
};

const WAIT_SLEEP_DURATION: Duration = Duration::from_secs(1);

pub fn watch(engine_args: EngineArgs) -> anyhow::Result<()> {
    initialize_logging();

    let game_name = engine_args.name.clone();

    // Shutdown signal
    let shutdown = Arc::new(AtomicBool::new(false));

    // Set break (Ctrl-C) handler.
    ctrlc::set_handler({
        let shutdown = shutdown.clone();

        move || {
            info!("Shutdown requested by user.");
            shutdown.store(true, Ordering::Release);
        }
    })
    .unwrap_or_else(|err| error!("Error setting Ctrl-C handler: {}", err));

    let ui = CliUiHandler;

    let engine = engine::run(engine_args, shutdown.clone(), ui)?;
    let mut engine_control = engine.control();

    // Wait for engine to start up
    while engine_control.state() != EngineState::Running {
        std::thread::sleep(WAIT_SLEEP_DURATION);
    }

    info!("Watching {game_name}");

    // Wait for shutdown signal
    while !shutdown.load(Ordering::Relaxed) {
        std::thread::sleep(WAIT_SLEEP_DURATION);
    }

    info!("Shutting down");

    // Request engine shutdown
    engine_control.shutdown();

    // Wait for engine to shut down
    while !engine.has_shut_down() {
        std::thread::sleep(WAIT_SLEEP_DURATION);
    }

    Ok(())
}

fn initialize_logging() {
    let subscriber = FmtSubscriber::builder()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .with_writer(std::io::stderr)
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("Setting default tracing subscriber failed!");
}

pub struct CliUiHandler;

impl StoolUiHandler for CliUiHandler {
    fn clear(self) -> anyhow::Result<()> {
        Ok(())
    }

    fn begin_backup(&mut self, name: &str) {
        info!("Creating backup: {name}");
    }

    fn end_backup(&mut self, success: bool) {
        if success {
            info!("Backup created successfully");
        } else {
            error!("Create backup failed");
        }
    }

    fn begin_staging(&mut self, _count: usize) {}

    fn begin_stage(&mut self, _name: &str) {}

    fn end_stage(&mut self) {}

    fn end_staging(&mut self) {}

    fn begin_compress(&mut self) {}

    fn end_compress(&mut self) {}

    fn begin_restore(&mut self, name: &str) {
        info!("Restoring backup: {name}");
    }

    fn end_restore(&mut self, success: bool) {
        if success {
            info!("Backup restored");
        } else {
            error!("Restore backup failed");
        }
    }

    fn begin_extract(&mut self) {}

    fn end_extract(&mut self) {}

    fn begin_restore_sp(&mut self, _name: &str) {}

    fn end_restore_sp(&mut self) {}
}

impl SyncUiHandler for CliUiHandler {
    fn begin_scan(&mut self) {}

    fn end_scan(&mut self) {}

    fn begin_prepare(&mut self) {}

    fn end_prepare(&mut self) {}

    fn begin_sync(&mut self, _op_count: usize) {}

    fn sync_progress(&mut self) {}

    fn end_sync(&mut self) {}

    fn begin_file(&mut self, _prefix: &str, _filename: &str, _size: u64) {}

    fn file_progress(&mut self, _bytes: u64) {}

    fn end_file(&mut self) {}
}
