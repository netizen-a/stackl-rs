use std::result;
pub enum Error {
	Unknown,
}

pub type Result<T> = result::Result<T, Error>;
