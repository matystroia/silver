use color_eyre::Result;

use crate::core::{Output, log::Logger, notify::Message};

pub trait OkOrNotify<T> {
    fn ok_or_notify(self) -> Option<T>;
}

impl<T> OkOrNotify<T> for Result<T> {
    fn ok_or_notify(self) -> Option<T> {
        match self {
            Ok(val) => Some(val),
            Err(err) => {
                Output::notify(Message::error(err.to_string()));
                Logger::error(err);
                None
            }
        }
    }
}
