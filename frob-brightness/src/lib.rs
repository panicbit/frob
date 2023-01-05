use std::{fs, path::{PathBuf, Path}, ffi::OsString, time::Duration};

use dbus::blocking::Connection;
use fauxpas::*;

#[derive(clap::Parser)]
pub enum Cli {
    Get,
    Set {
        #[clap(default_value = "1")]
        percent: u8,
    },
    #[clap(visible_alias = "up")]
    Increase {
        #[clap(default_value = "1")]
        amount: u8,
    },
    #[clap(visible_alias = "down")]
    Decrease {
        #[clap(default_value = "1")]
        amount: u8,
    },
    List,
}

pub fn run(cli: &Cli) -> Result<()> {
    match cli {
        Cli::Get => cmd_get(),
        Cli::Set { percent } => cmd_set(*percent),
        Cli::Increase { amount } => cmd_increase(*amount),
        Cli::Decrease { amount } => cmd_decrease(*amount),
        Cli::List => cmd_list(),
    }
}

fn cmd_get() -> Result<()> {
    let info = guess_best_backlight()?;

    println!("{:.0}", info.brightness_percent());

    Ok(())
}

fn cmd_set(percent: u8) -> Result<()> {
    let percent = percent.clamp(0, 100) as f32;
    let mut info = guess_best_backlight()?;

    info.set_brightness_percent(percent);
    info.save_brightness()?;

    Ok(())
}

fn cmd_increase(amount: u8) -> Result<()> {
    let amount = amount.clamp(0, 100) as f32;
    let mut info = guess_best_backlight()?;
    let new_percent = info.brightness_percent() + amount;

    info.set_brightness_percent(new_percent);
    info.save_brightness()?;

    Ok(())
}

fn cmd_decrease(amount: u8) -> Result<()> {
    let amount = amount.clamp(0, 100) as f32;
    let mut info = guess_best_backlight()?;
    let new_percent = info.brightness_percent() - amount;

    info.set_brightness_percent(new_percent);
    info.save_brightness()?;

    Ok(())
}

fn guess_best_backlight() -> Result<BacklightInfo> {
    let infos = get_all_backlight_infos()?;
    let info = infos.into_iter()
        .next()
        .context("No backlights found")?;

    Ok(info)
}

fn cmd_list() -> Result<()> {
    let infos = get_all_backlight_infos()?;

    for info in infos {
        println!("device: {:?}", info.name);
        println!("    brightness: {} ({:.0}%)", info.brightness, info.brightness_percent());
        println!("    max_brightness: {}", info.max_brightness);
        println!("    actual_brightness: {}", info.actual_brightness);
    }

    Ok(())
}

fn get_all_backlight_infos() -> Result<Vec<BacklightInfo>> {
    let paths = backlight_device_paths()
        .context("failed to get backlight device paths")?;

    let mut infos = Vec::new();

    for path in paths {
        let info = get_backlight_info(&path)
            .with_context(|| anyhow!("failed to get backlight info from {:?}", path))?;

        infos.push(info);
    }

    Ok(infos)
}

fn backlight_device_paths() -> Result<Vec<PathBuf>> {
    let entries = fs::read_dir("/sys/class/backlight")
        .context("failed to enumerate backlight devices")?;
    let mut paths = Vec::new();

    for entry in entries {
        let entry = entry.context("failed to enumerate backlight device")?;

        paths.push(entry.path());
    }

    Ok(paths)
}

fn get_backlight_info(path: impl AsRef<Path>) -> Result<BacklightInfo> {
    let path = path.as_ref();

    Ok(BacklightInfo {
        path: path.to_owned(),
        name: path.file_name()
            .context("BUG: backlight path without final component")?
            .to_owned(),
        brightness: read_u32_from_file(path.join("brightness"))
            .context("failed to read brightness")?,
        max_brightness: read_u32_from_file(path.join("max_brightness"))
            .context("failed to read max brightness")?,
        actual_brightness: read_u32_from_file(path.join("actual_brightness"))
            .context("failed to read actual brightness")?,
    })
}

fn read_u32_from_file(path: impl AsRef<Path>) -> Result<u32> {
    let path = path.as_ref();
    let value = fs::read_to_string(path)
        .with_context(|| fauxpas!("failed to read {:?}", path))?
        .trim()
        .parse::<u32>()
        .with_context(|| fauxpas!("failed to parse content of {:?} as u32", path))?;

    Ok(value)
}

#[derive(Debug)]
struct BacklightInfo {
    path: PathBuf,
    name: OsString,
    brightness: u32,
    max_brightness: u32,
    actual_brightness: u32,
}

impl BacklightInfo {
    fn brightness_percent(&self) -> f32 {
        if self.max_brightness == 0 {
            return 0.;
        }

        self.brightness as f32 / self.max_brightness as f32 * 100.
    }

    fn set_brightness_percent(&mut self, percent: f32) {
        let percent = percent.clamp(0., 100.).round();
        let new_brightness = percent / 100. * self.max_brightness as f32;

        self.brightness = new_brightness as u32;

        // Brightness value 0 turns the screen completely black.
        // This is usually not desired by the user.
        // TODO: make configurable
        if self.brightness == 0 {
            self.brightness = 1;
        }
    }

    fn save_brightness(&self) -> Result<()> {
        // TODO: allow using sysfs OR dbus
        // let path = self.path.join("brightness");
        // let brightness = self.brightness.to_string();

        // fs::write(&path, brightness)
        //     .with_context(|| anyhow!("Failed to write brightness to {:?}", path))?;

        let device_name = self.name.to_str()
            .with_context(|| anyhow!("Device name is invalid utf-8: {:?}", self.name))?;

        let conn = Connection::new_system()
            .context("Failed to connect to system D-Bus")?;

        let destination = "org.freedesktop.login1";
        let path = "/org/freedesktop/login1/session/auto";
        let timeout = Duration::from_secs(3);
        let proxy = conn.with_proxy(destination, path, timeout);

        let interface = "org.freedesktop.login1.Session";
        let method = "SetBrightness";
        let device_class = "backlight";
        let brightness = self.brightness;
        let args = (device_class, device_name, brightness);
        let _result: () = proxy.method_call(interface, method, args)
            .context("Failed to set brightness via dbus")?;

        Ok(())
    }
}
