use fauxpas::*;
use structopt::StructOpt;

use frob_brightness::{Opt, run};

fn main() -> Result<()> {
    let opt = Opt::from_args();

    run(&opt)?;

    Ok(())
}
