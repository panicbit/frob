use clap::Parser;
use fauxpas::*;

use frob_volume::{Cli, run};

fn main() -> Result<()> {
    let args = Cli::parse();

    run(&args)?;

    Ok(())
}
