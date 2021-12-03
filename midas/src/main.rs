use anyhow::*;
use structopt::StructOpt;

#[derive(StructOpt)]
pub enum Opt {
    Monitor(midas_monitor::Opt),
    Volume(midas_volume::Opt),
}

fn main() -> Result<()> {
    let opt = Opt::from_args();

    match opt {
        Opt::Monitor(opt) => midas_monitor::run(&opt),
        Opt::Volume(opt) => midas_volume::run(&opt),
    }
}
