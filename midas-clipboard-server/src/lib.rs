use fauxpas::*;
use structopt::StructOpt;

mod cmd;
mod wrapper;

#[derive(StructOpt)]
pub enum Opt {
    Start,
}

pub fn run(opt: &Opt) -> Result<()> {
    match opt {
        Opt::Start => cmd::start(),
    }
}
