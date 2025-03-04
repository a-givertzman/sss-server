///
/// Error container
#[derive(Debug, Clone, PartialEq)]
pub struct StrErr(pub String);
// Error doesn't require you to implement any methods, but
// your type must also implement Debug and Display.
impl std::error::Error for StrErr {}
//
//
impl std::fmt::Display for StrErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Delegate to the Display impl for &str:
        self.0.fmt(f)
    }
}
//
//
impl From<String> for StrErr {
    fn from(value: String) -> Self {
        StrErr(value)
    }
}
//
//
impl From<&str> for StrErr {
    fn from(value: &str) -> Self {
        StrErr(value.to_owned())
    }
}
//
//
impl From<&str> for Box<StrErr> {
    fn from(val: &str) -> Self {
        Box::new(StrErr(val.to_owned()))
    }
}
