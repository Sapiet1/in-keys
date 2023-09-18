use std::str;

/// Represents various types of keyboard input events.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Key {
    /// Unknown or unrecognized key
    Unknown,
    /// Left arrow key
    ArrowLeft,
    /// Right arrow key
    ArrowRight,
    /// Up arrow key
    ArrowUp,
    /// Down arrow key
    ArrowDown,
    /// Enter key
    Enter,
    /// Escape key
    Escape,
    /// Backspace key
    Backspace,
    /// Home key
    Home,
    /// End key
    End,
    /// Tab key
    Tab,
    /// BackTab (Shift + Tab) key
    BackTab,
    /// Alt key
    Alt,
    /// Delete key
    Del,
    /// Shift key
    Shift,
    /// Insert key
    Insert,
    /// Page Up key
    PageUp,
    /// Page Down key
    PageDown,
    /// A printable character (UTF-8)
    Char(char),
}

impl From<&[u8]> for Key {
    fn from(value: &[u8]) -> Self {
        str::from_utf8(value)
            .ok()
            .and_then(|string| string.chars().next())
            .map_or(Key::Unknown, Key::Char)
    }
}