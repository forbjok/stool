use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use iocraft::{ElementExt as _, element};
use tracing::{error, info};

use crate::engine::{self, EngineArgs};
use crate::tui2::Example;

pub fn tui2(engine_args: EngineArgs) -> anyhow::Result<()> {
    //let ui = TuiUiHandler::new(app_state.clone());

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

    //let engine = engine::run(engine_args, shutdown.clone(), ui)?;

    //crate::tui::run(engine, app_state, shutdown)?;
    smol::block_on(element!(Example).fullscreen()).unwrap();

    Ok(())
}
