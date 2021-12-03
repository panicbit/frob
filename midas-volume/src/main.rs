use fauxpas::*;
use structopt::StructOpt;

use midas_volume::{Opt, run};

fn main() -> Result<()> {
    let opt = Opt::from_args();

    run(&opt)?;

    Ok(())
}
