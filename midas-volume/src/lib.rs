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
use libpulse_binding::volume::{ChannelVolumes, VolumeLinear, Volume};
use structopt::StructOpt;

#[derive(StructOpt)]
pub enum Opt {
    Get,
    #[structopt(visible_alias = "up")]
    Increase(VolumeChange),
    #[structopt(visible_alias = "down")]
    Decrease(VolumeChange),
}

#[derive(StructOpt)]
pub struct VolumeChange {
    #[structopt(default_value = "1")]
    /// the amount to change the volume by
    amount: u16,
}

pub fn run(opt: &Opt) -> Result<()> {
    match opt {
        Opt::Get => cmd_get(),
        Opt::Increase(opt) => cmd_increase(opt),
        Opt::Decrease(opt) => cmd_decrease(opt),
    }
}

fn cmd_get() -> Result<()> {
    let (mut mainloop, context) = connect_to_pulseaudio()?;

    let sink_info = get_sink_info_by_name(&mut mainloop, &context, "@DEFAULT_SINK@")
        .context("Failed to get sink info")?;

    let volume = sink_info.volume.max();
    let volume = VolumeLinear::from(volume);

    println!("{:.0}", volume.0 * 100.);

    Ok(())
}

fn cmd_increase(opt: &VolumeChange) -> Result<()> {
    let (mut mainloop, context) = connect_to_pulseaudio()?;
    let amount = (opt.amount as f64).clamp(0., 100.);

    let sink_info = get_sink_info_by_name(&mut mainloop, &context, "@DEFAULT_SINK@")
        .context("Failed to get sink info")?;

    let mut volume = sink_info.volume;

    for volume in volume.get_mut() {
        let mut new_volume = VolumeLinear::from(*volume);
        let value = &mut new_volume.0;

        *value *= 100.;
        // println!("Old volume: {:.2}", value);
        *value += amount;
        *value = value.round().clamp(0., 100.);
        // println!("New volume: {:.2}", value);
        *value /= 100.;

        *volume = Volume::from(new_volume);
    }

    set_sink_volume_by_name(&mut mainloop, &context, "@DEFAULT_SINK@", &volume)
        .context("Failed to set sink volume")?;

    Ok(())
}

fn cmd_decrease(opt: &VolumeChange) -> Result<()> {
    let (mut mainloop, context) = connect_to_pulseaudio()?;
    let amount = (opt.amount as f64).clamp(0., 100.);

    let sink_info = get_sink_info_by_name(&mut mainloop, &context, "@DEFAULT_SINK@")
        .context("Failed to get sink info")?;

    let mut volume = sink_info.volume;

    for volume in volume.get_mut() {
        let mut new_volume = VolumeLinear::from(*volume);
        let value = &mut new_volume.0;

        *value *= 100.;
        // println!("Old volume: {:.2}", value);
        *value -= amount;
        *value = value.round().clamp(0., 100.);
        // println!("New volume: {:.2}", value);
        *value /= 100.;

        *volume = Volume::from(new_volume);
    }

    set_sink_volume_by_name(&mut mainloop, &context, "@DEFAULT_SINK@", &volume)
        .context("Failed to set sink volume")?;

    Ok(())
}

fn connect_to_pulseaudio() -> Result<(Mainloop, Context)> {
        let mut mainloop = Mainloop::new()
        .context("Failed to create main loop")?;

    let mut proplist = Proplist::new()
        .context("Failed to create proplist")?;
    proplist.set_str(APPLICATION_NAME, "midas")
        .map_err(|()| anyhow!("Failed to set application name"))?;

    let mut context = Context::new_with_proplist(&*mainloop.borrow(), "MidasContext", &proplist)
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

fn set_sink_volume_by_name(mainloop: &mut Mainloop, context: &Context, name: &str, volume: &ChannelVolumes) -> Result<()> {
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

#[derive(Debug)]
struct SinkInfo {
    volume: ChannelVolumes,
}
