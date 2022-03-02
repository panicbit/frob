use anyhow::*;
use structopt::StructOpt;

#[derive(StructOpt)]
pub enum Opt {
    Brightness(midas_brightness::Opt),
    ClipboardServer(midas_clipboard_server::Opt),
    Monitor(midas_monitor::Opt),
    Volume(midas_volume::Opt),
}

fn main() -> Result<()> {
    let opt = Opt::from_args();

    match opt {
        Opt::Brightness(opt) => midas_brightness::run(&opt),
        Opt::ClipboardServer(opt) => midas_clipboard_server::run(&opt),
        Opt::Monitor(opt) => midas_monitor::run(&opt),
        Opt::Volume(opt) => midas_volume::run(&opt),
    }
}
