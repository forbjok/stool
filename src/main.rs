mod command;
mod config;
mod engine;
mod internal;
#[cfg(feature = "tui")]
mod tui;

use anyhow::Context;
use clap::Parser;
use engine::EngineArgs;

#[derive(Debug, Parser)]
#[clap(name = "stool", version = env!("CARGO_PKG_VERSION"), author = env!("CARGO_PKG_AUTHORS"))]
struct Opt {
    #[clap(subcommand)]
    command: Command,
}

#[derive(Debug, Parser)]
enum Command {
    #[clap(about = "Create a new game config")]
    New,
    #[cfg(feature = "tui")]
    #[clap(about = "Run game via stool")]
    RunGame {
        #[clap(help = "Game name")]
        name: String,

        #[clap(help = "Game command")]
        game_command: Vec<String>,
    },
    #[cfg(feature = "tui")]
    #[clap(about = "Run stool in TUI mode")]
    Tui {
        #[clap(help = "Game name")]
        name: String,
    },
    #[clap(about = "Run stool in watch mode")]
    Watch {
        #[clap(help = "Game name")]
        name: String,
    },
}

fn main() -> anyhow::Result<()> {
    let opt = Opt::parse();

    let config_path = self::config::main::get_default_config_path().context("Getting default config path")?;
    let game_config_path = config_path.join("games");

    let config = self::config::main::MainConfig::load_or_write_default_from_location(&config_path)?;

    let data_path = config.data_path;

    match opt.command {
        Command::New => command::new(&game_config_path),
        #[cfg(feature = "tui")]
        Command::RunGame { name, game_command } => {
            let engine_args = EngineArgs {
                name,
                game_config_path,
                data_path,
            };
            command::rungame(engine_args, game_command)
        }
        #[cfg(feature = "tui")]
        Command::Tui { name } => {
            let engine_args = EngineArgs {
                name,
                game_config_path,
                data_path,
            };

            command::tui(engine_args)
        }
        Command::Watch { name } => {
            let engine_args = EngineArgs {
                name,
                game_config_path,
                data_path,
            };

            command::watch(engine_args)
        }
    }?;

    Ok(())
}
