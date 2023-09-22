#[cfg(unix)]
pub(crate) use crate::streams::unix::Config;
#[cfg(windows)]
pub(crate) use crate::streams::windows::Config;

pub(crate) enum Flag {
    Echo,
    Line,
    NoEcho,
    NoLine,
}