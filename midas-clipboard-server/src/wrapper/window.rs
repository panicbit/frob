use std::sync::Arc;

use fauxpas::*;
use x11rb::connection::Connection as _;
use x11rb::protocol::xproto::{
    Atom, ConnectionExt as _, CreateWindowAux, PropMode, Timestamp, WindowClass,
};
use x11rb::rust_connection::RustConnection;
use x11rb::wrapper::ConnectionExt as _;
use x11rb::{COPY_DEPTH_FROM_PARENT, CURRENT_TIME};

use crate::wrapper::{PropertyValueRef, WindowId};

pub struct Window {
    id: WindowId,
    conn: Arc<RustConnection>,
}

impl Window {
    pub fn new_dummy(conn: &Arc<RustConnection>) -> Result<Self> {
        let screen = conn.setup().roots.first().context("No roots available")?;
        let id = conn.generate_id().context("Failed to generate window id")?;
        let id = WindowId::from(id);

        conn.create_window(
            COPY_DEPTH_FROM_PARENT,
            *id,
            screen.root,
            0,
            0,
            1,
            1,
            0,
            WindowClass::INPUT_OUTPUT,
            0,
            &CreateWindowAux::default(),
        )?
        .check()
        .context("Failed to create window")?;

        Ok(Self::from_id(conn, id))
    }

    pub fn from_id(conn: &Arc<RustConnection>, id: WindowId) -> Self {
        Self {
            id,
            conn: conn.clone(),
        }
    }

    pub fn id(&self) -> WindowId {
        self.id
    }

    pub fn set_title(&self, title: &str) -> Result<()> {
        self.set_property_str("_NET_WM_NAME", title)
    }

    pub fn set_class(&self, class: &str) -> Result<()> {
        self.set_property_bytes("WM_CLASS", class)
    }

    pub fn set_property_str(&self, property: impl AsRef<[u8]>, data: &str) -> Result<()> {
        self.set_property(property, "UTF8_STRING", PropertyValueRef::U8(data.as_ref()))
    }

    pub fn set_property_bytes(
        &self,
        property: impl AsRef<[u8]>,
        bytes: impl AsRef<[u8]>,
    ) -> Result<()> {
        self.set_property(property, "STRING", PropertyValueRef::U8(bytes.as_ref()))
    }

    pub fn set_property_atoms(&self, property: impl AsRef<[u8]>, atoms: &[Atom]) -> Result<()> {
        self.set_property(property, "ATOM", PropertyValueRef::U32(atoms.as_ref()))
    }

    fn set_property(
        &self,
        property: impl AsRef<[u8]>,
        type_: impl AsRef<[u8]>,
        value: PropertyValueRef,
    ) -> Result<()> {
        let atom_property = self
            .conn
            .intern_atom(false, property.as_ref())?
            .reply()?
            .atom;
        let atom_type = self.conn.intern_atom(false, type_.as_ref())?.reply()?.atom;

        let result = match value {
            PropertyValueRef::U8(value) => self.conn.change_property8(
                PropMode::REPLACE,
                *self.id(),
                atom_property,
                atom_type,
                value,
            ),
            PropertyValueRef::U16(value) => self.conn.change_property16(
                PropMode::REPLACE,
                *self.id(),
                atom_property,
                atom_type,
                value,
            ),
            PropertyValueRef::U32(value) => self.conn.change_property32(
                PropMode::REPLACE,
                *self.id(),
                atom_property,
                atom_type,
                value,
            ),
        };

        result?.check().context("Failed to set property")?;

        Ok(())
    }

    pub fn acquire_clipboard_now(&self) -> Result<()> {
        self.acquire_clipboard_at(CURRENT_TIME)
    }

    pub fn acquire_clipboard_at(&self, time: impl Into<Timestamp>) -> Result<()> {
        let selection = self.conn.intern_atom(false, b"CLIPBOARD")?.reply()?.atom;

        self.conn
            .set_selection_owner(self.id(), selection, time)?
            .check()?;

        Ok(())
    }
}
