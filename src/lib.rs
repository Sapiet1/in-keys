//! # Terminal Input Handling (In-Keys)
//!
//! This crate provides functions for handling input from the terminal, with a focus on providing
//! both synchronous and asynchronous options. It uses low-level system calls for efficient input
//! polling and processing. Minimal support for output streams exist as well.
//!
//! ## Synchronous Input
//!
//! Synchronous input functions allow for blocking operations where the program waits for user input.
//! These functions include reading a single key, processing escape sequences for special keys, and
//! reading strings of characters terminated by the Enter key.
//!
//! ## Asynchronous Input
//!
//! Asynchronous input functions use a polling mechanism to check for available input without blocking
//! the program's execution. These functions can be useful in scenarios where the program needs to
//! remain responsive to other tasks or events while still monitoring for user input.
//!
//! ## Example Usage
//!
//! ```rust,ignore
//! use in_keys::Terminal;
//! use std::time::Duration;
//! use tokio::time;
//!
//! let terminal = Terminal::new();
//!
//! // Synchronous input
//! let key = terminal.read_key(); // Blocking, waits for user input
//!
//! // Asynchronous input
//! let mut lock = terminal.lock_stdin().expect("Failed to connect with terminal");
//! let future_key = lock.read_key_future(); // Returns a future immediately
//! let result = time::timeout(Duration::from_secs(5), future_key).await;
//!
//! match result {
//!     Ok(Ok(key)) => println!("Received key: {:?}", key),
//!     Ok(Err(error)) => eprintln!("Error reading key: {}", error),
//!     Err(_) => eprintln!("Timed out waiting for input"),
//! }
//! ```
//!
//! ## Notes
//!
//! - This module utilizes low-level system calls and may not be portable across all platforms.
//!   Only Linux is truly supported as of the current moment.
//! - Care should be taken when using asynchronous input, as it may introduce additional complexity
//!   and overhead.

use crate::keys::Key;
use crate::streams::{StderrLock, StdinLock, StdoutLock, Streams};

pub mod streams;
pub mod keys;

const FAILED_WRITE: &str = "failed to write to stream";
const FAILED_READ: &str = "failed to read from stream";
const FAILED_CONNECT: &str = "failed to connect with attended input stream";

/// A struct representing the terminal interface for input and output operations.
/// Only simple operations are enabled; lock the respective stream for more methods.
#[derive(Debug, Default)]
pub struct Terminal {
    streams: Streams,
}

/// An enum representing the target output stream, which can be either standard output or standard error.
pub enum Target {
    Stdout,
    Stderr,
}

impl Terminal {
    /// Creates a new instance of the `Terminal` struct, initializing the underlying input and output streams.
    pub fn new() -> Self {
        let streams = Streams::new();
        Terminal { streams }
    }

    /// Locks the standard input stream, allowing for synchronous read operations.
    /// Returns [`Some(StdinLock)`] if successful, or [`None`] if locking the stream fails.
    pub fn lock_stdin(&self) -> Option<StdinLock> {
        self.streams.lock_stdin()
    }

    /// Locks the standard output stream, allowing for synchronous write operations.
    pub fn lock_stdout(&self) -> StdoutLock {
        self.streams.lock_stdout()
    }

    /// Locks the standard error stream, allowing for synchronous write operations.
    pub fn lock_stderr(&self) -> StderrLock {
        self.streams.lock_stderr()
    }

    /// Prints a string to the specified target stream.
    /// If the target is [`Target::Stderr`], the string is printed to the standard error stream.
    /// If the target is [`Target::Stdout`], the string is printed to the standard output stream.
    /// Panics if an error occurs during writing.
    pub fn print(&self, target: Target, str: &str) {
        match target {
            Target::Stderr => self.streams.lock_stderr().print(str),
            Target::Stdout => self.streams.lock_stdout().print(str),
        }
        .unwrap_or_else(|error| panic!("{}: {}", error, FAILED_WRITE))
    }

    /// Prints a string followed by a newline to the specified target stream.
    /// If the target is [`Target::Stderr`], the string is printed to the standard error stream.
    /// If the target is [`Target::Stdout`], the string is printed to the standard output stream.
    /// Panics if an error occurs during writing.
    pub fn println(&self, target: Target, str: &str) {
        match target {
            Target::Stderr => self.streams.lock_stderr().println(str),
            Target::Stdout => self.streams.lock_stdout().println(str),
        }
        .unwrap_or_else(|error| panic!("{}: {}", error, FAILED_WRITE))
    }

    /// Clears the screen by sending an escape sequence.
    /// This function sends the escape sequence to clear the entire screen.
    /// It moves the cursor to the top-left corner of the terminal.
    /// Panics if an error occurs during writing.
    pub fn clear(&self) {
        self.streams
            .lock_stdout()
            .clear()
            .unwrap_or_else(|error| panic!("{}: {}", error, FAILED_WRITE))
    }

    /// Hides the cursor in the terminal.
    /// This function sends the escape sequence to hide the cursor in the terminal.
    /// Panics if an error occurs during writing.
    pub fn hide(&self) {
        self.streams
            .lock_stdout()
            .hide()
            .unwrap_or_else(|error| panic!("{}: {}", error, FAILED_WRITE))
    }

    /// Shows the cursor in the terminal.
    /// This function sends the escape sequence to show the cursor in the terminal.
    /// Panics if an error occurs during writing.
    pub fn show(&self) {
        self.streams
            .lock_stdout()
            .show()
            .unwrap_or_else(|error| panic!("{}: {}", error, FAILED_WRITE))
    }

    /// Reads a single key from the standard input stream.
    /// Panics if an error occurs during reading.
    pub fn read_key(&self) -> Key {
        self.streams
            .lock_stdin()
            .expect(FAILED_CONNECT)
            .read_key()
            .unwrap_or_else(|error| panic!("{}: {}", error, FAILED_READ))
    }

    /// Reads a line of text from the standard input stream.
    /// Panics if an error occurs during reading.
    pub fn read_string(&self) -> String {
        self.streams
            .lock_stdin()
            .expect(FAILED_CONNECT)
            .read_string()
            .unwrap_or_else(|error| panic!("{}: {}", error, FAILED_READ))
    }
}