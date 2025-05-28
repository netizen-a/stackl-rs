use crate::diag::lex;
use std::fmt;

#[non_exhaustive]
#[derive(Debug, Clone, Copy)]
pub enum Punctuator {
	/// `[`
	LSquare,
	/// `]`
	RSquare,
	/// `(`
	LParen,
	/// `)`
	RParen,
	/// `{`
	LCurly,
	/// `}`
	RCurly,
	/// `.`
	Dot,
	/// `->
	Arrow,
	/// `++`
	PlusPlus,
	/// --
	MinusMinus,
	/// &
	Amp,
	/// `*`
	Star,
	/// `+`
	Plus,
	/// `-`
	Minus,
	/// `~`
	Tilde,
	/// `!`
	Bang,
	/// `/`
	FSlash,
	/// `%`
	Percent,
	/// `<<`
	LessLess,
	/// `>>`
	GreatGreat,
	/// `<`
	Less,
	/// `>`
	Great,
	/// `<=`
	LessEqual,
	/// `>=`
	GreatEqual,
	/// `==`
	EqualEqual,
	/// `!=`
	BangEqual,
	/// `^`
	Caret,
	/// `|`
	VBar,
	/// `&&`
	AmpAmp,
	/// `||`
	VBarVBar,
	/// `?`
	QMark,
	/// `:`
	Colon,
	/// `;`
	SemiColon,
	/// `...`
	Ellipsis,
	/// `=`
	Equal,
	/// `*=`
	StarEqual,
	/// `/=`
	FSlashEqual,
	/// `%=`
	PercentEqual,
	/// `+=`
	PlusEqual,
	/// `-=`
	MinusEqual,
	/// `<<=`
	LessLessEqual,
	/// `>>=`
	GreatGreatEqual,
	/// `&=`
	AmpEqual,
	/// `^=`
	CaretEqual,
	/// `|=`
	VBarEqual,
	/// `,`
	Comma,
	/// `#`
	Hash,
	/// `##`
	HashHash,
}

impl fmt::Display for Punctuator {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		use Punctuator::*;
		let symbol = match self {
			LSquare => "[",
			RSquare => "]",
			LParen => "(",
			RParen => ")",
			LCurly => "{",
			RCurly => "}",
			Dot => ".",
			Arrow => "->",
			PlusPlus => "++",
			MinusMinus => "--",
			Amp => "&",
			Star => "*",
			Plus => "+",
			Minus => "-",
			Tilde => "~",
			Bang => "!",
			FSlash => "/",
			Percent => "%",
			LessLess => "<<",
			GreatGreat => ">>",
			Less => "<",
			Great => ">",
			LessEqual => "<=",
			GreatEqual => ">=",
			EqualEqual => "==",
			BangEqual => "!=",
			Caret => "^",
			VBar => "|",
			AmpAmp => "&&",
			VBarVBar => "||",
			QMark => "?",
			Colon => ":",
			SemiColon => ";",
			Ellipsis => "...",
			Equal => "=",
			StarEqual => "*=",
			FSlashEqual => "/=",
			PercentEqual => "%=",
			PlusEqual => "+=",
			MinusEqual => "-=",
			LessLessEqual => "<<=",
			GreatGreatEqual => ">>=",
			AmpEqual => "&=",
			CaretEqual => "^=",
			VBarEqual => "|=",
			Comma => ",",
			Hash => "#",
			HashHash => "##",
		};
		write!(f, "{}", symbol)
	}
}

impl TryFrom<char> for Punctuator {
	type Error = lex::TryFromCharError;
	fn try_from(value: char) -> Result<Self, Self::Error> {
		match value {
			'[' => Ok(Self::LSquare),
			']' => Ok(Self::RSquare),
			'(' => Ok(Self::LParen),
			')' => Ok(Self::RParen),
			'{' => Ok(Self::LCurly),
			'}' => Ok(Self::RCurly),
			'?' => Ok(Self::QMark),
			',' => Ok(Self::Comma),
			'~' => Ok(Self::Tilde),
			';' => Ok(Self::SemiColon),
			_ => Err(lex::TryFromCharError),
		}
	}
}
