use fauxpas::*;
use structopt::StructOpt;

#[derive(StructOpt)]
pub enum Opt {
    Start
}

pub fn run(opt: &Opt) -> Result<()> {
    match opt {
        Opt::Start => cmd_start()
    }
}

fn cmd_start() -> Result<()> {

    Ok(())
}
