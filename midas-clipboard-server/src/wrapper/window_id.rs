use std::{fmt, ops};

use x11rb::protocol::xproto;

#[derive(Copy, Clone, Debug)]
pub struct WindowId(u32);

impl From<xproto::Window> for WindowId {
    fn from(id: xproto::Window) -> Self {
        Self(id)
    }
}

impl From<WindowId> for xproto::Window {
    fn from(id: WindowId) -> Self {
        id.0
    }
}

impl ops::Deref for WindowId {
    type Target = xproto::Window;

    fn deref(&self) -> &xproto::Window {
        &self.0
    }
}

impl fmt::Display for WindowId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl fmt::LowerHex for WindowId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl fmt::UpperHex for WindowId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}
