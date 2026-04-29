use std::fmt;

#[derive(Debug, Clone)]
pub struct Identifier(String);

impl Identifier {
    pub fn new(s: impl Into<String>) -> Result<Self, String> {
        let s = s.into();
        if s.is_empty() {
            return Err("Identifier cannot be empty".to_string());
        }
        Ok(Self(s))
    }
}

impl AsRef<str> for Identifier {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl TryFrom<&str> for Identifier {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::new(normalize_identifier(value))
    }
}

impl TryFrom<String> for Identifier {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::try_from(value.as_str())
    }
}

impl fmt::Display for Identifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

fn normalize_identifier(s: &str) -> String {
    let mut normalized = s.replace(['-', ' ', '.', '/'], "_");

    if normalized.chars().next().is_some_and(|c| c.is_numeric()) {
        normalized = format!("_{normalized}");
    }

    // Handle Swift reserved keywords
    match normalized.as_str() {
        // Keywords used in declarations
        "associatedtype" | "class" | "deinit" | "enum" | "extension" | "fileprivate" | "func"
        | "import" | "init" | "inout" | "internal" | "let" | "open" | "operator" | "private"
        | "precedencegroup" | "protocol" | "public" | "rethrows" | "static" | "struct"
        | "subscript" | "typealias" | "var" |
        // Keywords used in statements
        "break" | "case" | "catch" | "continue" | "default" | "defer" | "do" | "else"
        | "fallthrough" | "for" | "guard" | "if" | "in" | "repeat" | "return" | "throw"
        | "switch" | "where" | "while" |
        // Keywords used in expressions and types
        "Any" | "as" | "false" | "is" | "nil" | "self" | "Self" | "super" | "throws" | "true"
        | "try" |
        // Keywords reserved in particular contexts
        "Type" | "Protocol" => format!("`{normalized}`"),
        _ => normalized,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identifier_normalization() {
        assert_eq!(
            Identifier::try_from("test-name").unwrap().as_ref(),
            "test_name"
        );
        assert_eq!(
            Identifier::try_from("test.name").unwrap().as_ref(),
            "test_name"
        );
        assert_eq!(
            Identifier::try_from("test/name").unwrap().as_ref(),
            "test_name"
        );
    }

    #[test]
    fn test_reserved_keywords() {
        assert_eq!(Identifier::try_from("class").unwrap().as_ref(), "`class`");
        assert_eq!(Identifier::try_from("func").unwrap().as_ref(), "`func`");
        assert_eq!(
            Identifier::try_from("protocol").unwrap().as_ref(),
            "`protocol`"
        );
        assert_eq!(Identifier::try_from("self").unwrap().as_ref(), "`self`");
        assert_eq!(Identifier::try_from("Type").unwrap().as_ref(), "`Type`");
    }

    #[test]
    fn test_identifiers_starting_with_numbers() {
        assert_eq!(Identifier::try_from("123").unwrap().as_ref(), "_123");
        assert_eq!(Identifier::try_from("1st").unwrap().as_ref(), "_1st");
        assert_eq!(
            Identifier::try_from("2factor").unwrap().as_ref(),
            "_2factor"
        );
        assert_eq!(Identifier::try_from("3-way").unwrap().as_ref(), "_3_way");
    }
}
