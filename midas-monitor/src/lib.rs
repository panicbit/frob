use anyhow::*;
use structopt::StructOpt;

mod cycle;

#[derive(StructOpt)]
pub enum Opt {
    Cycle(cycle::Opt),
}

pub fn run(opt: &Opt) -> Result<()> {
    match opt {
        Opt::Cycle(opt) => cycle::run(opt),
    }
}
