use std::fmt;

mod r#enum;
mod ident;
mod intersection;
mod object;
mod union;

pub use r#enum::*;
pub use ident::*;
pub use intersection::*;
pub use object::*;
pub use union::*;

#[derive(Debug, Clone, Copy)]
pub struct Indent(pub usize);

impl fmt::Display for Indent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{: <indent$}", "", indent = self.0 * 4)
    }
}

#[derive(Debug, Clone)]
pub struct Comment(pub String);

impl Comment {
    pub fn lines(&self) -> impl Iterator<Item = &str> {
        self.0.trim().lines().map(str::trim)
    }
}
