use anyhow::*;
use structopt::StructOpt;

#[derive(StructOpt)]
pub enum Opt {
    Monitor(midas_monitor::Opt),
}

fn main() -> Result<()> {
    let opt = Opt::from_args();

    match opt {
        Opt::Monitor(opt) => midas_monitor::run(&opt),
    }
}
