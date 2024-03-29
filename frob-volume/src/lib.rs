use std::borrow::Borrow;
use std::cell::RefCell;
use std::rc::Rc;

use fauxpas::{*, Context as _};
use libpulse_binding::callbacks::ListResult;
use libpulse_binding::context::{self, Context};
use libpulse_binding::mainloop::standard::{Mainloop, IterateResult};
use libpulse_binding::operation;
use libpulse_binding::proplist::Proplist;
use libpulse_binding::proplist::properties::APPLICATION_NAME;
use libpulse_binding::volume::{ChannelVolumes, Volume};

#[derive(clap::Parser)]
pub enum Cli {
    Get,
    Set { percent: u8 },
    #[clap(visible_alias = "up")]
    Increase(VolumeChange),
    #[clap(visible_alias = "down")]
    Decrease(VolumeChange),
    ToggleMute,
}

#[derive(clap::Args)]
pub struct VolumeChange {
    #[clap(default_value = "1")]
    /// the amount to change the volume by
    amount: u8,
}

pub fn run(cli: &Cli) -> Result<()> {
    match cli {
        Cli::Get => cmd_get(),
        Cli::Set { percent } => cmd_set(*percent),
        Cli::Increase(args) => cmd_increase(args),
        Cli::Decrease(args) => cmd_decrease(args),
        Cli::ToggleMute => cmd_toggle_mute(),
    }
}

fn cmd_get() -> Result<()> {
    let (mut mainloop, context) = connect_to_pulseaudio()?;

    let sink_info = get_sink_info_by_name(&mut mainloop, &context, "@DEFAULT_SINK@")
        .context("Failed to get sink info")?;

    let volume = sink_info.volume.max();

    println!("{:.0}", volume.percent());

    Ok(())
}

fn cmd_set(percent: u8) -> Result<()> {
    let (mut mainloop, context) = connect_to_pulseaudio()?;
    let percent = (percent as f32).clamp(0., 100.);

    let sink_info = get_sink_info_by_name(&mut mainloop, &context, "@DEFAULT_SINK@")
        .context("Failed to get sink info")?;

    let mut volume = sink_info.volume;

    for volume in volume.get_mut() {
        volume.set_percent(percent);
    }

    set_sink_volume_by_name(&mut mainloop, &context, "@DEFAULT_SINK@", &volume)
        .context("Failed to set sink volume")?;

    Ok(())
}

fn cmd_increase(args: &VolumeChange) -> Result<()> {
    let (mut mainloop, context) = connect_to_pulseaudio()?;
    let amount = (args.amount as f32).clamp(0., 100.);

    let sink_info = get_sink_info_by_name(&mut mainloop, &context, "@DEFAULT_SINK@")
        .context("Failed to get sink info")?;

    let mut volume = sink_info.volume;

    for volume in volume.get_mut() {
        let new_percent = (volume.percent() + amount)
            .round()
            .clamp(0., 100.);

        volume.set_percent(new_percent);
    }

    set_sink_volume_by_name(&mut mainloop, &context, "@DEFAULT_SINK@", &volume)
        .context("Failed to set sink volume")?;

    Ok(())
}

fn cmd_decrease(args: &VolumeChange) -> Result<()> {
    let (mut mainloop, context) = connect_to_pulseaudio()?;
    let amount = (args.amount as f32).clamp(0., 100.);

    let sink_info = get_sink_info_by_name(&mut mainloop, &context, "@DEFAULT_SINK@")
        .context("Failed to get sink info")?;

    let mut volume = sink_info.volume;

    for volume in volume.get_mut() {
        let new_percent = (volume.percent() - amount)
            .round()
            .clamp(0., 100.);

        volume.set_percent(new_percent);
    }

    set_sink_volume_by_name(&mut mainloop, &context, "@DEFAULT_SINK@", &volume)
        .context("Failed to set sink volume")?;

    Ok(())
}

fn cmd_toggle_mute() -> Result<()> {
    let (mut mainloop, context) = connect_to_pulseaudio()?;
    let sink_name = "@DEFAULT_SINK@";

    let sink_info = get_sink_info_by_name(&mut mainloop, &context, sink_name)
        .context("Failed to get sink info")?;

    let mute = !sink_info.mute;

    set_sink_mute_by_name(&mut mainloop, &context, sink_name, mute)
        .context("Failed to set sink mute flag")?;

    Ok(())
}

fn connect_to_pulseaudio() -> Result<(Mainloop, Context)> {
        let mut mainloop = Mainloop::new()
        .context("Failed to create main loop")?;

    let mut proplist = Proplist::new()
        .context("Failed to create proplist")?;
    proplist.set_str(APPLICATION_NAME, "frob")
        .map_err(|()| anyhow!("Failed to set application name"))?;

    let mut context = Context::new_with_proplist(&*mainloop.borrow(), "FrobContext", &proplist)
        .context("Failed to create context")?;

    context.connect(None, context::FlagSet::NOFLAGS, None)
        .context("Failed to connect to pulseaudio")?;

    loop {
        match mainloop.iterate(true) {
            IterateResult::Quit(_) |
            IterateResult::Err(_) => {
                bail!("Iterate state was not success, quitting...");
            },
            IterateResult::Success(_) => {},
        }

        match context.borrow().get_state() {
            context::State::Ready => { break; },
            context::State::Failed |
            context::State::Terminated => {
                bail!("Context state failed/terminated, quitting...");
            },
            _ => {},
        }
    }

    Ok((mainloop, context))
}

fn get_sink_info_by_name(mainloop: &mut Mainloop, context: &Context, name: &str) -> Result<SinkInfo> {
    let introspector = context.introspect();
    let sink_info_result = Rc::new(RefCell::new(None::<Result<SinkInfo, ()>>));

    let operation = introspector.get_sink_info_by_name(name, {
        let sink_info_result = sink_info_result.clone();

        move |list| {
            let mut sink_info_result = sink_info_result.borrow_mut();

            match list {
                ListResult::Item(sink_info) => *sink_info_result = Some(Result::Ok(SinkInfo {
                    volume: sink_info.volume,
                    mute: sink_info.mute,
                })),
                ListResult::End => {},
                ListResult::Error => *sink_info_result = Some(Err(())),
            }
        }
    });

    loop {
        match mainloop.iterate(true) {
            IterateResult::Quit(_) |
            IterateResult::Err(_) => {
                bail!("Iterate state was not success, quitting...");
            },
            IterateResult::Success(_) => {},
        }

        match operation.get_state() {
            operation::State::Running => {},
            operation::State::Done => break,
            operation::State::Cancelled => break,
        }
    }

    let sink_info = Rc::try_unwrap(sink_info_result)
        .map_err(|_| anyhow!("Failed to regain ownership of sink info result"))?
        .into_inner()
        .with_context(|| anyhow!("Sink not found: {}", name))?
        .map_err(|_: ()| fauxpas!("Faled to get sink info"))?;

    Ok(sink_info)
}

fn set_sink_volume_by_name(
    mainloop: &mut Mainloop,
    context: &Context,
    name: &str,
    volume: &ChannelVolumes,
) -> Result<()> {
    let mut introspector = context.introspect();
    let operation = introspector.set_sink_volume_by_name(name, volume, None);

    loop {
        match mainloop.iterate(true) {
            IterateResult::Quit(_) |
            IterateResult::Err(_) => {
                bail!("Iterate state was not success, quitting...");
            },
            IterateResult::Success(_) => {},
        }

        match operation.get_state() {
            operation::State::Running => {},
            operation::State::Done => break,
            operation::State::Cancelled => break,
        }
    }

    Ok(())
}

fn set_sink_mute_by_name(
    mainloop: &mut Mainloop,
    context: &Context,
    name: &str,
    mute: bool,
) -> Result<()> {
    let mut introspector = context.introspect();
    let operation = introspector.set_sink_mute_by_name(name, mute, None);

    loop {
        match mainloop.iterate(true) {
            IterateResult::Quit(_) |
            IterateResult::Err(_) => {
                bail!("Iterate state was not success, quitting...");
            },
            IterateResult::Success(_) => {},
        }

        match operation.get_state() {
            operation::State::Running => {},
            operation::State::Done => break,
            operation::State::Cancelled => break,
        }
    }

    Ok(())
}

trait VolumeExt {
    fn percent(&self) -> f32;
    fn set_percent(&mut self, percent: f32);
}

impl VolumeExt for Volume {
    fn percent(&self) -> f32 {
        self.0 as f32 / Volume::NORMAL.0 as f32 * 100.
    }

    fn set_percent(&mut self, percent: f32) {
        self.0 = (percent / 100. * Volume::NORMAL.0 as f32) as u32;
    }
}

#[derive(Debug)]
struct SinkInfo {
    volume: ChannelVolumes,
    mute: bool,
}
