use std::{
    mem::MaybeUninit,
    io::{StdoutLock, StdinLock, BufRead},
    os::windows::io::AsRawHandle,
    io::{Error as IoError, ErrorKind, Result as IoResult},
};

use windows_sys::Win32::{
    Foundations,
    System::Console,
};

use crate::{
    keys,
    streams::config::Flag,
};

const FAILURE: isize = 0;

fn io_error(c_call: impl FnOnce() -> Foundations::BOOL) -> IoResult<()> {
    match c_call() {
        FAILURE => Err(IoError::last_os_error()),
        _ => Ok(()),
    }
}

fn flush_input(lock: &mut StdinLock<'static>) {
    unsafe {
    }
}

pub(crate) struct Config<'a> {
    pub(super) lock: &'a mut StdinLock<'static>,
    original: Console::CONSOLE_MODE,
    flush: bool,
}

impl<'a> Config<'a> {
    pub(super) fn set(lock: &'a mut StdinLock<'static>, flush: bool, flags: &[Flag]) -> Self {
        unsafe {
            let mut mode = MaybeUninit::uninit();
            io_error(|| Console::GetConsoleMode(lock.as_raw_handle(), mode.as_mut_ptr())).expect("failed reading flags");

            let mut mode = mode.assume_init();
            let original = mode;

            for flag in flags {
                match flag {
                    Flag::Line => mode |= Console::ENABLE_LINE_INPUT,
                    Flag::Echo => mode |= Console::ENABLE_ECHO_INPUT,
                    Flag::NoLine => mode &= !Console::ENABLE_LINE_INPUT,
                    Flag::NoEcho => mode &= !Console::ENABLE_ECHO_INPUT,
                }
            }

            if flush { flush_input(lock); }
            io_error(|| Console::SetConsoleMode(lock.as_raw_handle(), mode)).expect("failed setting flags");
            Config { lock, original, flush }
        }
    }
}