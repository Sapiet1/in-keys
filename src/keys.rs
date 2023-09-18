// `keys.rs` follows the general architecture from `console-rs/console`, which is licensed under the MIT License.
//
// The MIT License (MIT)
//
// Copyright (c) 2017 Armin Ronacher <armin.ronacher@active-4.com>
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

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