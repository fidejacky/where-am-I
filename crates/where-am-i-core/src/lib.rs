mod error;
mod location;
pub mod provider;

pub use error::Error;
pub use location::{Location, Source};
pub use provider::Provider;

pub fn locate() -> Result<Location, Error> {
    provider::ip::IpInfo::default().locate()
}
