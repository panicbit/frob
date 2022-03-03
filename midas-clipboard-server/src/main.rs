use fauxpas::*;
use structopt::StructOpt;

use midas_clipboard_server::{run, Opt};

fn main() -> Result<()> {
    let opt = Opt::from_args();

    run(&opt)?;

    Ok(())
}
