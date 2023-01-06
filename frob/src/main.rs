use std::io;

use anyhow::*;
use clap::{CommandFactory, Parser};
use clap_complete::Shell;

#[derive(Parser)]
pub enum Cli {
    #[clap(subcommand)]
    Brightness(frob_brightness::Cli),
    #[clap(subcommand)]
    Monitor(frob_monitor::Cli),
    #[clap(subcommand)]
    Volume(frob_volume::Cli),
    #[clap(hide = true)]
    Completions {
        shell: clap_complete::Shell,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli {
        Cli::Brightness(cli) => frob_brightness::run(&cli),
        Cli::Monitor(cli) => frob_monitor::run(&cli),
        Cli::Volume(cli) => frob_volume::run(&cli),
        Cli::Completions { shell } => generate_completions(shell),
    }
}

fn generate_completions(shell: Shell) -> Result<()> {
    clap_complete::generate(shell, &mut Cli::command(), "frob", &mut io::stdout());

    Ok(())
}
