use anyhow::*;
use structopt::StructOpt;

#[derive(StructOpt)]
pub enum Opt {
    Brightness(frob_brightness::Opt),
    Monitor(frob_monitor::Opt),
    Volume(frob_volume::Opt),
}

fn main() -> Result<()> {
    let opt = Opt::from_args();

    match opt {
        Opt::Brightness(opt) => frob_brightness::run(&opt),
        Opt::Monitor(opt) => frob_monitor::run(&opt),
        Opt::Volume(opt) => frob_volume::run(&opt),
    }
}
