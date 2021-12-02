use anyhow::*;
use structopt::StructOpt;

use midas_monitor::{Opt, run};

fn main() -> Result<()> {
    let opt = Opt::from_args();

    run(&opt)?;

    Ok(())
}
