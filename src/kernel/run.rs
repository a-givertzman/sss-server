///
/// Application entity execution
pub trait Run {
    fn run(&mut self) -> Result<(), String>;
}
