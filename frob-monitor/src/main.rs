use anyhow::*;
use clap::Parser;
use frob_monitor::{Cli, run};

fn main() -> Result<()> {
    let cli = Cli::parse();

    run(&cli)?;

    Ok(())
}
