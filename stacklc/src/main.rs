use std::{fs, process::ExitCode};

mod cli;
mod lex;

fn main() -> ExitCode {
    let args = cli::Args::parse();
    let Ok(file) = fs::File::open(&args.in_file) else {
        eprintln!("Failed to open file `{}`", args.in_file.to_str().unwrap());
        return ExitCode::FAILURE;
    };

    ExitCode::SUCCESS
}

#[cfg(test)]
mod tests {
    use super::lex;
    use std::{fs, io::Read};
    #[test]
    fn lexical_analysis() {
        let mut file = fs::File::open("tests/test01.c").unwrap();
        let mut buf = String::new();
        file.read_to_string(&mut buf).unwrap();
        let lexer = lex::lexer::Lexer::new(&buf, 0);
        for token in lexer {
            match token {
                Ok(pp_token) => {
                    println!("ok: {pp_token:?}");
                }
                Err(lex_err) => {
                    eprintln!("err: {lex_err:?}");
                }
            }
        }
    }
}
