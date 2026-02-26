use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "jaxon")]
#[command(about = "A CLI for managing developer products and gamepasses")]
#[command(
    long_about = "jaxon is a command-line tool for creating, syncing, and managing developer products and game passes."
)]
pub struct Args {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Initialize a new jaxon project in the current directory
    #[command(long_about = "Initialize a new jaxon project in the current directory.

This command creates the base configuration files required for jaxon
to operate. By default, it generates:

  - jaxon.toml       Project configuration file
  - products.json    Initial product definition file

You can use the --minimal flag to generate only the configuration file
and skip creating product-related files.")]
    Init {
        /// Create only the toml file
        #[arg(short, long)]
        minimal: bool,
    },

    /// Sync local product definitions to Roblox
    #[command(
        long_about = "Synchronize products defined in the local products.json file\nwith Roblox developer products and game passes."
    )]
    Sync,
}
