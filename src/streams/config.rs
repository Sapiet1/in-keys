pub(crate) use crate::streams::unix::Config;

pub(crate) enum Flag {
    Echo,
    Canonical,
    NotEcho,
    NotCanonical,
}