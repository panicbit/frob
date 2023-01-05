use clap::Parser;
use fauxpas::*;

use frob_brightness::{Cli, run};

fn main() -> Result<()> {
    let cli = Cli::parse();

    run(&cli)?;

    Ok(())
}
