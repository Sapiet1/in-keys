// `unix.rs` follows the general architecture from `console-rs/console`, which is licensed under the MIT License.
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

use std::{
    mem::MaybeUninit,
    io::{StdoutLock, StdinLock, BufRead},
    os::fd::AsRawFd,
    io::{Error as IoError, ErrorKind, Result as IoResult},
};

use crate::{
    keys::Key,
    streams::config::Flag,
};

// Constant representing a successful system call result.
const SUCCESS: i32 = 0;

// Executes a C function call that returns a libc integer and handles any potential error.
fn io_error(c_call: impl FnOnce() -> libc::c_int) -> IoResult<()> {
    match c_call() {
        SUCCESS => Ok(()), // Call was successful, return Ok(())
        _ => Err(IoError::last_os_error()), // Call failed, return the last OS error (ERRNO)
    }
}

// Attains the window size of the terminal, in (`row`, `column`) notation.
pub fn size(lock: &StdoutLock) -> Option<(usize, usize)> {
    // Safety: `ioctl` is appropriately used.
    unsafe {
        let mut size: libc::winsize = std::mem::zeroed();

        #[allow(clippy::useless_conversion)]
        libc::ioctl(lock.as_raw_fd(), libc::TIOCGWINSZ.into(), &mut size);
        (size.ws_row > 0 && size.ws_col > 0).then_some((size.ws_row as usize, size.ws_col as usize))
    }
}

// Polls the standard input stream for available input.
// `timeout` is the time, in milliseconds, to wait for input. 0 is non-blocking and negative is forever blocking.
// The returned `bool` indicating whether there is input available [`true`] or not [`false`].
fn poll_input(lock: &StdinLock, timeout: i32) -> IoResult<bool> {
    // Safety: Count for `fds` is properly managed.
    unsafe {
        let mut fds = libc::pollfd {
            fd: lock.as_raw_fd(),  // Standard input file descriptor
            events: libc::POLLIN,  // Interested in read events
            revents: 0,            // Placeholder for returned events
        };

        // Call the `poll` system call, using a closure to pass the pointer to `fds`.
        // The `min` function is used to ensure a successful result (>= 0) is always 0.
        io_error(|| libc::poll(&mut fds as *mut _, 1, timeout).min(SUCCESS))?;

        // Check if POLLIN event occurred and return result
        Ok(fds.revents & libc::POLLIN == libc::POLLIN)
    }
}

// Reads a fixed-size byte array from standard input, specified by a const-generic.
// `_lock` refers to the `StdinLock` for correctness, and `timeout` is the timeout in milliseconds.
// 0 is non-blocking and negative is forever blocking.
// If input is available, an `IoResult` containing an `Option` of a byte array with size `N` is returned.
// If no input is available within the specified timeout, `Ok(None)` is returned.
fn read_bytes<const N: usize>(lock: &mut StdinLock, timeout: i32) -> IoResult<Option<[u8; N]>> {
    // Check if input is available, return None if not
    if !poll_input(lock, timeout)? { return Ok(None); }

    // Special case for zero-sized array, return filled array of zeros
    if N == 0 { return Ok(Some([0; N])); }

    // Create a buffer to hold the read bytes
    let mut buffer = [0; N];

    // Use unsafe Rust to call the `read` system call, populating the buffer
    // Safety: Valid `fd` and buffer.
    let read = unsafe { libc::read(lock.as_raw_fd(), buffer.as_mut_ptr().cast(), N) };

    // Match on the result of the read and the buffer contents
    match (read, buffer) {
        (0, _) => Err(IoError::from(ErrorKind::UnexpectedEof)), // Return UnexpectedEof if no bytes were read
        (_, buffer) if buffer[0] == b'\x03' => Err(IoError::from(ErrorKind::Interrupted)), // Return Interrupted if Ctrl+C was pressed
        (_, buffer) => Ok(Some(buffer)), // Return the read bytes
    }
}

// This function processes the input received from the user.
fn process_key(lock: &mut StdinLock, timeout: i32) -> IoResult<Option<Key>> {
    // Try to read one byte from the input
    match read_bytes::<1>(lock, timeout)? {
        // If an escape character (0x1b) is received and there's more input available
        Some([b'\x1b']) if poll_input(lock, 0)? => {
            // Match on the next two bytes to determine special key combinations
            let key = match read_bytes::<2>(lock, 0)? {
                Some([b'[', b'A']) => return Ok(Some(Key::ArrowUp)),
                Some([b'[', b'B']) => return Ok(Some(Key::ArrowDown)),
                Some([b'[', b'C']) => return Ok(Some(Key::ArrowRight)),
                Some([b'[', b'D']) => return Ok(Some(Key::ArrowLeft)),
                Some([b'[', b'H']) => return Ok(Some(Key::Home)),
                Some([b'[', b'F']) => return Ok(Some(Key::End)),
                Some([b'[', b'Z']) => return Ok(Some(Key::BackTab)),
                Some([b'[', b'1']) => Ok(Some(Key::Home)),
                Some([b'[', b'2']) => Ok(Some(Key::Insert)),
                Some([b'[', b'3']) => Ok(Some(Key::Del)),
                Some([b'[', b'4']) => Ok(Some(Key::End)),
                Some([b'[', b'5']) => Ok(Some(Key::PageUp)),
                Some([b'[', b'6']) => Ok(Some(Key::PageDown)),
                Some([b'[', b'7']) => Ok(Some(Key::Home)),
                Some([b'[', b'8']) => Ok(Some(Key::End)),
                _ => return Ok(Some(Key::Unknown)),
            };

            // Check for a tilde (~) character indicating the end of an escape sequence
            match read_bytes::<1>(lock, 0)? {
                Some([b'~']) => key,
                _ => Ok(Some(Key::Unknown)),
            }
        },
        // If only an escape character (0x1b) is received
        Some([b'\x1b']) => Ok(Some(Key::Escape)),
        // If a byte other than an escape character is received
        Some([byte]) => match byte {
            // Handle UTF-8 multi-byte sequences
            byte if byte & 224_u8 == 192_u8 => {
                let Some([second]) = read_bytes::<1>(lock, 0)? else {
                    return Ok(Some(Key::Unknown));
                };

                Ok(Some((&[byte, second][..]).into()))
            },
            byte if byte & 240_u8 == 224_u8 => {
                let Some([second, third]) = read_bytes::<2>(lock, 0)? else {
                    return Ok(Some(Key::Unknown));
                };

                Ok(Some((&[byte, second, third][..]).into()))
            },
            byte if byte & 248u8 == 240u8 => {
                let Some([second, third, fourth]) = read_bytes::<3>(lock, 0)? else {
                    return Ok(Some(Key::Unknown));
                };

                Ok(Some((&[byte, second, third, fourth][..]).into()))
            },
            // Handle special control characters
            b'\n' | b'\r' => Ok(Some(Key::Enter)),
            b'\x7f' => Ok(Some(Key::Backspace)),
            b'\t' => Ok(Some(Key::Tab)),
            b'\x01' => Ok(Some(Key::Home)),
            b'\x05' => Ok(Some(Key::End)),
            b'\x08' => Ok(Some(Key::Backspace)),
            // Handle regular printable characters
            byte => Ok(Some(Key::Char(byte as char))),
        },
        // If no input is received
        None => Ok(None),
    }
}

// This function reads a single key from the terminal input.
pub(super) fn read_key(lock: &mut StdinLock, timeout: i32) -> IoResult<Option<Key>> {
    process_key(lock, timeout)
}

// This function reads a string of characters from the terminal input.
pub(super) fn read_string(lock: &mut StdinLock, timeout: i32) -> IoResult<Option<String>> {
    if poll_input(lock, timeout)? {
        let mut buffer = String::new();
        lock.read_line(&mut buffer)?;

        Ok(Some(buffer))
    } else {
        Ok(None)
    }
}

pub(crate) struct Config<'a> {
    pub(super) lock: &'a mut StdinLock<'static>,
    original: libc::termios,
    flush: bool,
}

impl<'a> Config<'a> {
    pub(super) fn set(lock: &'a mut StdinLock<'static>, flush: bool, flags: &[Flag]) -> Self {
        // Safety: `termios` is properly handled
        unsafe {
            // Initialize termios struct
            let mut termios = MaybeUninit::uninit();
            // Theoretically, the call will never fail.
            io_error(|| libc::tcgetattr(lock.as_raw_fd(), termios.as_mut_ptr())).unwrap();

            // Get the initialized termios struct
            let mut termios = termios.assume_init();
            // Store the original settings for later restoration
            let original = termios;

            // Set flags
            for flag in flags {
                match flag {
                    Flag::Canonical => termios.c_lflag |= libc::ICANON,
                    Flag::Echo => termios.c_lflag |= libc::ECHO,
                    Flag::NotCanonical => termios.c_lflag &= !libc::ICANON,
                    Flag::NotEcho => termios.c_lflag &= !libc::ECHO,
                }
            }

            // Apply the modified termios settings
            let action = if flush { libc::TCSAFLUSH } else { libc::TCSADRAIN };
            // This particular call will also, theoretically, never fail.
            io_error(|| libc::tcsetattr(lock.as_raw_fd(), action, &termios)).unwrap();
            Config { lock, original, flush }
        }
    }
}

impl<'a> Drop for Config<'a> {
    fn drop(&mut self) {
        // Safety: `termios` is properly handled
        unsafe {
            // Restore the original termios settings
            let action = if self.flush { libc::TCSAFLUSH } else { libc::TCSANOW };
            // Call will theoretically never fail.
            io_error(|| libc::tcsetattr(self.lock.as_raw_fd(), action, &self.original)).unwrap();
        }
    }
}
