
x11rb::atom_manager! {
    pub Atoms: AtomsCookie {
        UTF8_STRING,
        STRING,
        ATOM,
        WM_CLASS,
        _NET_WM_NAME,
        CLIPBOARD,
        text_plain_utf8: b"text/plain;charset=utf-8",
        text_plain: b"text/plain",
    }
}
