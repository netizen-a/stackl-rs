mod cli;
mod lex;

fn main() {
    let _args = cli::Args::parse();
    println!("args: {:?}", _args);
}
