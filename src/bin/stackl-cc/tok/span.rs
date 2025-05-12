#[derive(Debug, Clone)]
pub struct Span {
    pub location: (isize, isize),
    pub file_key: usize,
    pub leading_tabs: usize,
    pub leading_spaces: usize,
}
impl Span {
    pub fn print_whitespace(&self) {
        print!("{}", "\t".repeat(self.leading_tabs));
        print!("{}", " ".repeat(self.leading_spaces));
    }
}

pub trait Spanned {
    fn span(&self) -> Span;
    fn set_span(&mut self, span: Span);
}
