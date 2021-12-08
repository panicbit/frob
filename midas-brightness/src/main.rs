use fauxpas::*;
use structopt::StructOpt;

use midas_brightness::{Opt, run};

fn main() -> Result<()> {
    let opt = Opt::from_args();

    run(&opt)?;

    Ok(())
}
