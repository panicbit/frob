use std::process::{Command, Stdio};

use anyhow::*;
use serde::Deserialize;

#[derive(clap::Args)]
pub struct Args {
    #[clap(
        long = "reverse",
        short = 'r',
        visible_alias = "rev",
    )]
    /// reverse cycle direction
    reverse: bool,
    #[clap(
        long = "attempts",
        default_value = "1",
    )]
    /// number of cycle attempts (workaround for an i3 bug)
    attempts: usize,
}

pub fn run(args: &Args) -> Result<()> {
    let mut workspaces = get_workspaces()
        .context("Failed to get workspaces")?;

    workspaces.sort_by_key(|workspace| (workspace.rect.y, workspace.rect.x));

    if args.reverse {
        workspaces.reverse();
    }

    let next_workspace = match find_next_workspace(&workspaces) {
        Some(next_workspace) => next_workspace,
        None => return Ok(()),
    };

    for _ in 0..args.attempts {
        switch_to_workspace(next_workspace)
            .context("Failed to switch workspace")?;
    }

    Ok(())
}

fn find_next_workspace(workspaces: &[Workspace]) -> Option<&Workspace> {
    workspaces
        .iter()
        .filter(|workspace| workspace.visible)
        .cycle()
        .skip_while(|workspace| !workspace.focused)
        .take(workspaces.len())
        .nth(1)
}

fn get_workspaces() -> Result<Vec<Workspace>> {
    let output = Command::new("i3-msg")
        .args(&["-t", "get_workspaces"])
        .stderr(Stdio::inherit())
        .output()
        .context("Failed to run i3-msg")?;

    ensure!(output.status.success(), "i3-msg returned with status {}", output.status);

    let workspaces = serde_json::from_slice::<Vec<Workspace>>(&output.stdout)
        .context("Failed to parse i3-msg output")?;

    Ok(workspaces)
}

fn switch_to_workspace(workspace: &Workspace) -> Result<()> {
    let status = Command::new("i3-msg")
        .args(&["-t", "command", "workspace", &workspace.name])
        .status()
        .context("Failed to run i3-msg")?;

    ensure!(status.success(), "i3-msg returned with status {}", status);

    Ok(())
}

// https://github.com/i3/i3/blob/729452448b5fe5aa63c31e2ca7fefc86c73b553b/docs/ipc#L199-L235
#[derive(Deserialize)]
struct Workspace {
    // id: usize,
    // num: u8,
    name: String,
    visible: bool,
    focused: bool,
    rect: Rect,
    // output: String,
    // urgent: bool,
}

#[derive(Deserialize)]
struct Rect {
    x: u32,
    y: u32,
    // width: u32,
    // height: u32,
}
