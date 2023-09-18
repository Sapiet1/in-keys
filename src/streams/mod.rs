use std::io::{
    self,
    Stderr, Stdout, Stdin,
    Write,
    Result as IoResult,
    IsTerminal,
};

use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
    time::Duration,
};

use derived_deref::{Deref, DerefMut};

mod unix;

use crate::keys::Key;
use unix::{read_key, read_string};

// This struct represents the standard streams: stderr, stdout, and stdin.
#[derive(Debug)]
pub(super) struct Streams {
    // The standard error stream.
    stderr: Stderr,
    // The standard output stream.
    stdout: Stdout,
    // The standard input stream, if available (i.e., in a user-attended terminal).
    stdin: Option<Stdin>,
}

/// A wrapper for the standard input lock, allowing for synchronous read operations.
#[derive(Debug, Deref, DerefMut)]
pub struct StdinLock(io::StdinLock<'static>);

// This macro generates asynchronous read functions with associated documentation.
macro_rules! read_future {
    // For each provided set of identifiers, types, and associated documentation...
    ( $( $docs:literal | $read_future:ident as $future_read:ident -> $ret:ty ),* $( , )? ) => { $(
        // Generate a function with the specified identifier and return type,
        // along with its associated documentation.
        #[doc = $docs]
        pub fn $read_future(&mut self) -> impl Future<Output = IoResult<$ret>> + '_ {
            // Define a struct for the asynchronous read operation.
            struct ReadFuture<'a> {
                lock: &'a mut StdinLock,
            }

            // Implement the Future trait for the asynchronous read operation.
            impl<'a> Future for ReadFuture<'a> {
                type Output = IoResult<$ret>;

                // Define how the future is polled.
                fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
                    // Call the provided asynchronous read function and handle the result.
                    match $future_read(self.lock, 0)? {
                        Some(out) => Poll::Ready(Ok(out)),
                        None => {
                            // If no data is available, wake the task for later polling.
                            cx.waker().wake_by_ref();
                            Poll::Pending
                        },
                    }
                }
            }

            // Return an instance of the asynchronous read future.
            ReadFuture { lock: self }
        }
    )* };
}

// This macro generates read functions with timeout support and associated documentation.
macro_rules! read_or_timeout {
    // For each provided set of identifiers, types, and associated documentation...
    ( $( $docs:literal | $read_or_timeout:ident as $timeout_read:ident -> $ret:ty ),* $( , )? ) => { $(
        // Generate a function with the specified identifier and return type,
        // along with its associated documentation.
        #[doc = $docs]
        pub fn $read_or_timeout(
            &mut self,
            timeout: Duration,
        ) -> IoResult<Option<$ret>>
        {
            // Convert the timeout duration to milliseconds.
            let mut timeout = timeout.as_millis();
            loop {
                // If the remaining timeout is greater than the maximum i32 value...
                if timeout > i32::MAX as u128 {
                    // Call the provided read function with the maximum timeout value.
                    match $timeout_read(self, i32::MAX)? {
                        // If data is available, return it as a [`Some`] variant.
                        Some(read) => return Ok(Some(read)),
                        // Otherwise, decrement the remaining timeout by the maximum value.
                        None => timeout -= i32::MAX as u128,
                    }
                } else {
                    // If the remaining timeout is within the i32 range...
                    // Call the provided read function with the converted timeout value.
                    return $timeout_read(self, timeout as i32);
                }
            }
        }
    )* };
}

impl StdinLock {
    /// Reads a single key from the standard input stream.
    pub fn read_key(&mut self) -> IoResult<Key> {
        read_key(self, -1).map(Option::unwrap)
    }

    /// Reads a line of text from the standard input stream.
    pub fn read_string(&mut self) -> IoResult<String> {
        read_string(self, -1).map(Option::unwrap)
    }

    read_or_timeout! {
        "Reads a key with an optional timeout." |
        read_key_or_timeout as read_key -> Key,
        "Reads a line of text with an optional timeout." |
        read_string_or_timeout as read_string -> String,
    }

    read_future! {
        "Reads a key asynchronously." |
        read_key_future as read_key -> Key,
        "Reads a line of text asynchronously." |
        read_string_future as read_string -> String,
    }
}

/// A wrapper for the standard output lock.
#[derive(Debug, Deref, DerefMut)]
pub struct StdoutLock(io::StdoutLock<'static>);

// Internal function for printing a string to the specified writer.
fn print_<const LN: bool>(writer: &mut impl Write, str: &str) -> IoResult<()> {
    writer.write_all(str.as_bytes())?;

    if LN { writer.write_all(&[b'\n']) }
    else { writer.flush() }
}

impl StdoutLock {
    /// Prints the specified string to the standard output.
    pub fn print(&mut self, str: &str) -> IoResult<()> {
        print_::<false>(&mut **self, str)
    }

    /// Prints the specified string to the standard output, followed by a newline character.
    pub fn println(&mut self, str: &str) -> IoResult<()> {
        print_::<true>(&mut **self, str)
    }

    /// Clears the screen by sending an escape sequence.
    pub fn clear(&mut self) -> IoResult<()> {
        const CLEAR_SCREEN: &str = "\r\x1b[2J\r\x1b[H";
        self.print(CLEAR_SCREEN)
    }

    /// Clears the screen from the cursor position to the end of the screen.
    pub fn clear_to_end(&mut self) -> IoResult<()> {
        const CLEAR_TO_END: &str = "\x1b[J";
        self.print(CLEAR_TO_END)
    }

    /// Clears the screen from the beginning to the cursor position.
    pub fn clear_to_beginning(&mut self) -> IoResult<()> {
        const CLEAR_TO_BEGINNING: &str = "\x1b[1J";
        self.print(CLEAR_TO_BEGINNING)
    }

    /// Clears the current line from the cursor position to the end of the line.
    pub fn clear_line_to_end(&mut self) -> IoResult<()> {
        const CLEAR_LINE_TO_END: &str = "\x1b[K";
        self.print(CLEAR_LINE_TO_END)
    }

    /// Clears the current line from the beginning to the cursor position.
    pub fn clear_line_to_beginning(&mut self) -> IoResult<()> {
        const CLEAR_LINE_TO_BEGINNING: &str = "\x1b[1K";
        self.print(CLEAR_LINE_TO_BEGINNING)
    }

    /// Moves the cursor to the specified row and column.
    pub fn move_cursor(&mut self, row: usize, col: usize) -> IoResult<()> {
        let move_cursor = format!("\x1b[{};{}H", row, col);
        self.print(&move_cursor)
    }

    /// Moves the cursor up by a specified number of rows.
    pub fn move_cursor_up(&mut self, rows: usize) -> IoResult<()> {
        let move_up = format!("\x1b[{}A", rows);
        self.print(&move_up)
    }

    /// Moves the cursor down by a specified number of rows.
    pub fn move_cursor_down(&mut self, rows: usize) -> IoResult<()> {
        let move_down = format!("\x1b[{}B", rows);
        self.print(&move_down)
    }

    /// Moves the cursor forward (right) by a specified number of columns.
    pub fn move_cursor_forward(&mut self, cols: usize) -> IoResult<()> {
        let move_forward = format!("\x1b[{}C", cols);
        self.print(&move_forward)
    }

    /// Moves the cursor backward (left) by a specified number of columns.
    pub fn move_cursor_backward(&mut self, cols: usize) -> IoResult<()> {
        let move_backward = format!("\x1b[{}D", cols);
        self.print(&move_backward)
    }

    /// Hides the cursor in the terminal.
    pub fn hide(&mut self) -> IoResult<()> {
        const HIDE_CURSOR: &str = "\x1b[?25l";
        self.print(HIDE_CURSOR)
    }

    /// Shows the cursor in the terminal.
    pub fn show(&mut self) -> IoResult<()> {
        const SHOW_CURSOR: &str = "\x1b[?25h";
        self.print(SHOW_CURSOR)
    }
}

/// A wrapper for the standard error lock.
#[derive(Debug, Deref, DerefMut)]
pub struct StderrLock(io::StderrLock<'static>);

impl StderrLock {
    /// Prints the specified string to the standard error stream.
    pub fn print(&mut self, str: &str) -> IoResult<()> {
        print_::<false>(&mut **self, str)
    }

    /// Prints the specified string to the standard error stream, followed by a newline character.
    pub fn println(&mut self, str: &str) -> IoResult<()> {
        print_::<true>(&mut **self, str)
    }
}

impl Streams {
    // Creates a new Streams instance with the standard input, output, and error streams.
    pub(super) fn new() -> Self {
        let stderr = io::stderr();
        let stdout = io::stdout();
        let stdin = Some(io::stdin()).filter(Stdin::is_terminal);

        Streams { stderr, stdout, stdin }
    }

    // Locks the standard output stream, providing a controlled interface for writing.
    pub(super) fn lock_stdout(&self) -> StdoutLock {
        let lock = self.stdout.lock();
        StdoutLock(lock)
    }

    // Locks the standard error stream, providing a controlled interface for writing.
    pub(super) fn lock_stderr(&self) -> StderrLock {
        let lock = self.stderr.lock();
        StderrLock(lock)
    }

    // Attempts to lock the standard input stream if it is associated with a user-attended terminal.
    pub(super) fn lock_stdin(&self) -> Option<StdinLock> {
        self.stdin
            .as_ref()
            .map(Stdin::lock)
            .map(StdinLock)
    }
}

impl Default for Streams {
    fn default() -> Self {
        Streams::new()
    }
}