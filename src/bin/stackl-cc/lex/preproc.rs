use super::error::*;
use super::lexer as lex;
use super::tok::Punctuator;
use super::tok::PunctuatorTerminal;
use super::tok::{self, Spanned};
use std::collections::{HashMap, VecDeque};
use std::io::BufReader;
use std::io::Read;
use std::{fs, io, path};
use tok::PPToken;
use tok::Token;

#[derive(Debug)]
pub enum ParseError {
    LexError(LexicalError),
    IOError(io::Error),
}

impl From<LexicalError> for ParseError {
    fn from(value: LexicalError) -> Self {
        Self::LexError(value)
    }
}

#[derive(Debug)]
pub struct MacroArgs {
    ident_list: Vec<tok::Identifier>,
    ellipsis: bool,
}

#[derive(Debug)]
pub enum MacroDef {
    Object(Vec<PPToken>),
    Function {
        args: MacroArgs,
        replacement_list: Vec<PPToken>,
    },
}

pub struct Preprocessor {
    file_map: bimap::BiHashMap<usize, path::PathBuf>,
    macros: HashMap<String, MacroDef>,
    stdout: i32,
    pp_tokens: VecDeque<PPToken>,
    is_newline: bool,
    is_preproc: bool,
}

impl Preprocessor {
    pub fn new<P>(file_path: P, stdout: i32) -> Self
    where
        P: AsRef<path::Path>,
    {
        let mut file_map = bimap::BiHashMap::new();
        file_map.insert(0, file_path.as_ref().to_owned());
        Self {
            file_map,
            macros: HashMap::new(),
            stdout,
            pp_tokens: VecDeque::new(),
            is_newline: true,
            is_preproc: false,
        }
    }
    pub fn parse(&mut self) -> Result<Vec<tok::Token>, Vec<ParseError>> {
        let file_path = self.file_map.get_by_left(&0).unwrap();
        let file = fs::File::open(file_path)
            .map_err(ParseError::IOError)
            .map_err(|e| vec![e])?;
        let mut reader = BufReader::new(file);
        let mut buf = String::new();
        reader
            .read_to_string(&mut buf)
            .map_err(ParseError::IOError)
            .map_err(|e| vec![e])?;
        drop(reader);
        let lexer = lex::Lexer::new(&buf, 0);

        let mut errors: Vec<ParseError> = vec![];
        for result in lexer {
            match result {
                Ok(pp_token) => self.pp_tokens.push_back(pp_token),
                Err(lex_error) => errors.push(lex_error.into()),
            }
        }

        let mut tokens = vec![];
        while let Some(pp_token) = self.pp_tokens.pop_front() {
            match self.tokenize(pp_token) {
                Ok(mut processed_tokens) => tokens.append(&mut processed_tokens),
                Err(mut processed_errors) => errors.append(&mut processed_errors),
            }
        }

        if !errors.is_empty() {
            Err(errors)
        } else {
            Ok(tokens)
        }
    }
    fn tokenize(&mut self, pp_token: PPToken) -> Result<Vec<tok::Token>, Vec<ParseError>> {
        match pp_token {
            PPToken::NewLine(token) => {
                self.pp_newline(token);
                Ok(vec![])
            }
            PPToken::Comment(token) => {
                if self.stdout > 1 {
                    token.span().print_whitespace();
                    print!("{}", token.name);
                }
                Ok(vec![])
            }
            PPToken::Identifier(token) => {
                if self.is_preproc {
                    self.directive(token)?;
                    Ok(vec![])
                } else if self.macros.contains_key(&token.name) {
                    self.expand_macro(token)
                } else {
                    if self.stdout > 0 {
                        token.span().print_whitespace();
                        print!("{}", token);
                    }
                    if let Ok(kw) = tok::Keyword::try_from(token.clone()) {
                        Ok(vec![Token::Keyword(kw)])
                    } else {
                        Ok(vec![Token::Identifier(token)])
                    }
                }
            }
            PPToken::Punctuator(token) => {
                if let tok::PunctuatorTerminal::Hash = token.term {
                    self.is_preproc = self.is_newline;
                    self.is_newline = false;
                    Ok(vec![])
                } else {
                    if self.stdout > 0 {
                        token.span().print_whitespace();
                        print!("{}", token.term);
                    }
                    Ok(vec![Token::Punctuator(token)])
                }
            }
            PPToken::StringLiteral(token) => {
                self.is_newline = false;
                if self.stdout > 0 {
                    token.span().print_whitespace();
                    print!("{}", token.name);
                }
                Ok(vec![Token::StringLiteral(token)])
            }
            PPToken::CharacterConstant(token) => {
                self.is_newline = false;
                if self.stdout > 0 {
                    token.span().print_whitespace();
                    print!("{}", token.name);
                }
                Ok(vec![Token::Constant(tok::Constant::Character(token))])
            }
            PPToken::PPNumber(token) => {
                self.is_newline = false;
                let token = Token::try_from(token).map_err(|e| vec![e.into()])?;
                if self.stdout > 0 {
                    token.span().print_whitespace();
                    print!("{token}");
                }
                Ok(vec![token])
            }
            PPToken::HeaderName(token) => todo!("header-name = {token:?}"),
        }
    }
    fn pp_newline(&mut self, token: tok::NewLine) {
        if self.stdout > 0 && !token.is_deleted {
            token.span().print_whitespace();
            println!();
        }
        self.is_preproc = false;
        self.is_newline = true;
    }
    fn expand_macro(&mut self, ident: tok::Identifier) -> Result<Vec<tok::Token>, Vec<ParseError>> {
        let macro_def = self.macros.get(&ident.name).unwrap();
        match macro_def {
            MacroDef::Object(replacement_list) => {
                for pp_token in replacement_list {
                    self.pp_tokens.push_front(pp_token.clone());
                }
                Ok(vec![])
            }
            MacroDef::Function {
                args,
                replacement_list,
            } => {
                // args.ident_list
                todo!("macro arguments")
            }
        }
    }
    fn directive(&mut self, ident: tok::Identifier) -> Result<(), Vec<ParseError>> {
        match ident.name.as_str() {
            "define" => self.pp_define(ident.span).map_err(|e| vec![e]),
            "undef" => self.pp_undef(ident.span).map_err(|e| vec![e]),
            "include" => self.pp_include(),
            _ => todo!("undefined directive: `{}`", ident.name),
        }
    }
    fn pp_define(&mut self, last_span: tok::Span) -> Result<(), ParseError> {
        let pp_token = match self.pp_tokens.pop_front() {
            Some(pp_token) => pp_token,
            None => {
                let (_, hi) = last_span.location;
                let span = tok::Span {
                    location: (hi + 1, hi + 1),
                    file_key: last_span.file_key,
                    leading_tabs: 0,
                    leading_spaces: 0,
                };
                return Err(LexicalError {
                    kind: LexicalErrorKind::UnexpectedEof,
                    span,
                }
                .into());
            }
        };
        let tok::PPToken::Identifier(ident) = pp_token else {
            return Err(LexicalError {
                kind: LexicalErrorKind::InvalidToken,
                span: pp_token.span(),
            }
            .into());
        };

        let mut args = MacroArgs {
            ident_list: vec![],
            ellipsis: false,
        };
        let mut is_obj = true;

        if let Some(PPToken::Punctuator(Punctuator {
            term: PunctuatorTerminal::LParen,
            ..
        })) = self.pp_tokens.front()
        {
            is_obj = false;
            // consume `(`
            self.pp_tokens.pop_front();
            let mut expected_ident = true;
            let mut expected_rparen = false;
            while let Some(pp_token) = self.pp_tokens.pop_front() {
                match pp_token {
                    PPToken::Identifier(ident) => {
                        if !expected_ident || expected_rparen {
                            return Err(LexicalError {
                                kind: LexicalErrorKind::InvalidToken,
                                span: ident.span,
                            }
                            .into());
                        }
                        if expected_ident {
                            args.ident_list.push(ident);
                            expected_ident = false;
                        }
                    }
                    PPToken::Punctuator(Punctuator {
                        term: PunctuatorTerminal::RParen,
                        ..
                    }) => {
                        break;
                    }
                    PPToken::Punctuator(Punctuator {
                        term: PunctuatorTerminal::Comma,
                        span,
                    }) => {
                        if expected_ident || expected_rparen {
                            return Err(LexicalError {
                                kind: LexicalErrorKind::InvalidToken,
                                span,
                            }
                            .into());
                        }
                        expected_ident = true;
                    }
                    PPToken::Punctuator(Punctuator {
                        term: PunctuatorTerminal::Ellipsis,
                        span,
                    }) => {
                        if expected_ident || expected_rparen {
                            return Err(LexicalError {
                                kind: LexicalErrorKind::InvalidToken,
                                span,
                            }
                            .into());
                        }
                        expected_rparen = true;
                    }
                    other => {
                        return Err(LexicalError {
                            kind: LexicalErrorKind::InvalidToken,
                            span: other.span(),
                        }
                        .into());
                    }
                }
            }
        }

        let mut replacement_list = vec![];
        while let Some(pp_token) = self.pp_tokens.pop_front() {
            if let PPToken::NewLine(token) = pp_token {
                self.pp_newline(token);
                break;
            }
            replacement_list.push(pp_token);
        }
        let macro_def = if is_obj {
            MacroDef::Object(replacement_list)
        } else {
            MacroDef::Function {
                args,
                replacement_list,
            }
        };
        self.macros.insert(ident.name, macro_def);
        Ok(())
    }
    fn pp_undef(&mut self, last_span: tok::Span) -> Result<(), ParseError> {
        let pp_token = match self.pp_tokens.pop_front() {
            Some(pp_token) => pp_token,
            None => {
                let (_, hi) = last_span.location;
                let span = tok::Span {
                    location: (hi + 1, hi + 1),
                    file_key: last_span.file_key,
                    leading_tabs: 0,
                    leading_spaces: 0,
                };
                return Err(LexicalError {
                    kind: LexicalErrorKind::UnexpectedEof,
                    span,
                }
                .into());
            }
        };
        let tok::PPToken::Identifier(ident) = pp_token else {
            return Err(LexicalError {
                kind: LexicalErrorKind::InvalidToken,
                span: pp_token.span(),
            }
            .into());
        };
        let _ = self.macros.remove(ident.name.as_str());
        while let Some(pp_token) = self.pp_tokens.pop_front() {
            if let PPToken::NewLine(newline) = pp_token {
                self.pp_newline(newline);
                break;
            }
        }

        Ok(())
    }
    fn pp_include(&mut self) -> Result<(), Vec<ParseError>> {
        self.is_preproc = false;
        self.is_newline = true;
        match self.pp_tokens.pop_front() {
            Some(PPToken::HeaderName(header)) => {
                if header.is_std {
                    todo!()
                } else {
                    let file_path: path::PathBuf = path::PathBuf::from(header.name);
                    let file = fs::File::open(&file_path)
                        .map_err(ParseError::IOError)
                        .map_err(|e| vec![e])?;
                    let mut reader = BufReader::new(file);
                    let mut buf = String::new();
                    reader
                        .read_to_string(&mut buf)
                        .map_err(ParseError::IOError)
                        .map_err(|e| vec![e])?;
                    drop(reader);

                    let file_key = if let Some(file_key) = self.file_map.get_by_right(&file_path) {
                        *file_key
                    } else {
                        let file_key = self.file_map.len();
                        self.file_map.insert(file_key, file_path);
                        file_key
                    };

                    let lexer = lex::Lexer::new(&buf, file_key);
                    let mut errors = vec![];
                    let mut pp_token_list = vec![];
                    for pp_token in lexer.into_iter() {
                        match pp_token {
                            Ok(token) => pp_token_list.push(token),
                            Err(error) => errors.push(error.into()),
                        }
                    }
                    for pp_token in pp_token_list.into_iter().rev() {
                        self.pp_tokens.push_front(pp_token)
                    }

                    if errors.is_empty() {
                        Ok(())
                    } else {
                        Err(errors)
                    }
                }
            }
            _ => todo!("header-name"),
        }
    }
}
