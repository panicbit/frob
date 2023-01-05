use anyhow::*;

mod cycle;

#[derive(clap::Parser)]
pub enum Cli {
    Cycle(cycle::Args),
}

pub fn run(cli: &Cli) -> Result<()> {
    match cli {
        Cli::Cycle(args) => cycle::run(args),
    }
}
