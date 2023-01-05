use anyhow::*;
use clap::Parser;

#[derive(Parser)]
pub enum Cli {
    #[clap(subcommand)]
    Brightness(frob_brightness::Cli),
    #[clap(subcommand)]
    Monitor(frob_monitor::Cli),
    #[clap(subcommand)]
    Volume(frob_volume::Cli),
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli {
        Cli::Brightness(cli) => frob_brightness::run(&cli),
        Cli::Monitor(cli) => frob_monitor::run(&cli),
        Cli::Volume(cli) => frob_volume::run(&cli),
    }
}
