use std::sync::Arc;

use fauxpas::*;
use x11rb::NONE;
use x11rb::connection::Connection;
use x11rb::protocol::xproto::{
    ConnectionExt as _, EventMask, SelectionNotifyEvent, SelectionRequestEvent,
    SELECTION_NOTIFY_EVENT,
};
use x11rb::protocol::Event;
use x11rb::rust_connection::RustConnection;

use crate::wrapper::{Atoms, Window};

const WINDOW_TITLE: &str = "midas-clipboard-server";
const WINDOW_CLASS: &str = WINDOW_TITLE;

pub fn start() -> Result<()> {
    let (conn, _screen_num) = x11rb::connect(None).context("Failed to connect to X server")?;
    let conn = Arc::new(conn);
    let atoms = Atoms::new(&*conn)?
        .reply()
        .context("Failed to create atoms")?;
    let window = Window::new_dummy(&conn, &atoms).context("Failed to create window")?;

    window
        .set_title(WINDOW_TITLE)
        .context("Failed to set window title")?;

    window
        .set_class(WINDOW_CLASS)
        .context("Failed to set window class")?;

    window
        .acquire_clipboard_now()
        .context("Failed to acquire clipboard")?;

    println!("Window id: 0x{:08x}", window.id());

    while let Ok(event) = conn.wait_for_event() {
        match event {
            Event::SelectionRequest(event) => {
                println!("We got asked to send the selection! {event:#?}");
                handle_selection_request(&conn, &atoms, &event)?;
            }
            Event::SelectionClear(event) => {
                println!("We lost ownership of the selection! {event:#?}");
            }
            _ => println!("{event:?}"),
        }
    }

    Ok(())
}

fn handle_selection_request(
    conn: &Arc<RustConnection>,
    atoms: &Atoms,
    event: &SelectionRequestEvent,
) -> Result<()> {
    let mut response = SelectionNotifyEvent {
        response_type: SELECTION_NOTIFY_EVENT,
        sequence: 0,
        time: event.time,
        requestor: event.requestor,
        selection: event.selection,
        target: event.target,
        property: event.property,
    };

    let requestor = Window::from_id(conn, atoms, event.requestor.into());

    let target_name = conn.get_atom_name(event.target)?.reply()?.name;
    let target_name = String::from_utf8_lossy(&target_name);

    let property_name = conn.get_atom_name(event.property)?.reply()?.name;
    let property_name = String::from_utf8_lossy(&property_name);

    println!(
        "Sending data to window 0x{:08x}, property {}, target {}",
        event.requestor, property_name, target_name
    );

    match &*target_name {
        "TARGETS" => {
            let targets = &[
                atoms.UTF8_STRING,
                atoms.text_plain_utf8,
                atoms.text_plain,
            ];

            requestor.set_property_atoms(event.property, targets)?;
        }
        "UTF8_STRING" | "text/plain;charset=utf-8" | "text/plain" => {
            let payload = "Hello from Rust! 🦀";

            requestor.set_property_str(event.property, payload)?;
        }
        _ => {
            response.property = NONE;
            println!("unsupported target: {}", target_name)
        },
    };

    conn.send_event(true, event.requestor, EventMask::NO_EVENT, response)?
        .check()?;

    Ok(())
}
