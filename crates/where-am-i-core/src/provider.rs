use crate::{Error, Location};

pub mod ip;

pub trait Provider {
    fn name(&self) -> &'static str;

    fn locate(&self) -> Result<Location, Error>;
}
