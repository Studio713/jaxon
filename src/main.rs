use anyhow::Result;
use clap::Parser;
use cli::{Args, Command};

mod cli;
mod code;
mod commands;
mod config;
mod lock;
mod products;
mod roblox;

fn main() -> Result<()> {
    dotenvy::dotenv().ok();
    let args = Args::parse();

    match args.command {
        Command::Sync => commands::sync::run()?,
        Command::Init { minimal } => commands::init::run(minimal)?,
    }

    Ok(())
}
