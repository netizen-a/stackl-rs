use std::result;

#[derive(Debug)]
pub enum Error {
	Unknown,
}

pub type ResultTriple<Tok, Loc> = result::Result<(Loc, Tok, Loc), Error>;
