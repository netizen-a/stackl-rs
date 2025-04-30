use std::{fs, process::ExitCode};

mod cli;
mod lex;

fn main() -> ExitCode {
    let args = cli::Args::parse();
    let Ok(file) = fs::File::open(&args.in_file) else {
        eprintln!("Failed to open file `{}`", args.in_file.to_str().unwrap());
        return ExitCode::FAILURE;
    };

    return ExitCode::SUCCESS;
}

#[cfg(test)]
mod tests {
    use super::lex;
    use super::lex::elements as tok;
    use std::{fs, io::Read};
    #[test]
    fn lexical_analysis() {
        let mut file = fs::File::open("tests/test02.txt").unwrap();
        let mut buf = String::new();
        file.read_to_string(&mut buf).unwrap();
        let lexer = lex::lexer::Lexer::new(&buf, 0);
        let tokens: Vec<tok::PreprocessingToken> = lexer.map(|t| t.unwrap()).collect();
        println!("tokens: {tokens:?}");
    }
}
