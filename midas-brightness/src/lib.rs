use std::{fs, path::{PathBuf, Path}, ffi::OsString};

use fauxpas::*;
use structopt::StructOpt;

#[derive(StructOpt)]
pub enum Opt {
    List,
}

pub fn run(opt: &Opt) -> Result<()> {
    match opt {
        Opt::List => cmd_list(),
    }
}

fn cmd_list() -> Result<()> {
    let paths = backlight_device_paths()
        .context("failed to get backlight device paths")?;
    
    for path in paths {
        let info = get_backlight_info(path)
            .context("failed to get backlight info")?;

        println!("device: {:?}", info.name);
        println!("    brightness: {}", info.brightness);
        println!("    max_brightness: {}", info.max_brightness);
        println!("    actual_brightness: {}", info.actual_brightness);
    }

    Ok(())
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
    let value = std::fs::read_to_string(path)
        .with_context(|| fauxpas!("failed to read {:?}", path))?
        .trim()
        .parse::<u32>()
        .with_context(|| fauxpas!("failed to parse content of {:?} as u32", path))?;
    
    Ok(value)
}

#[derive(Debug)]
struct BacklightInfo {
    name: OsString,
    brightness: u32,
    max_brightness: u32,
    actual_brightness: u32,
}
