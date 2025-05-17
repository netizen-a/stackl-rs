use std::{result, sync::mpsc::Receiver};

use crate::tok::{self, Token};

pub enum SyntaxError {
	Unknown,
}

pub type Result<T> = result::Result<T, SyntaxError>;

pub struct SyntaxParser {
	rcv_tokens: Receiver<tok::Result<Token>>,
}

impl SyntaxParser {
	pub fn new(rcv_tokens: Receiver<tok::Result<Token>>) -> Self {
		Self { rcv_tokens }
	}
}
