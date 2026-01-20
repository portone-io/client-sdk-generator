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

    // Handle Kotlin reserved keywords
    match normalized.as_str() {
        "abstract" | "annotation" | "as" | "break" | "by" | "catch" | "class" | "companion"
        | "const" | "constructor" | "continue" | "crossinline" | "data" | "delegate" | "do"
        | "dynamic" | "else" | "enum" | "expect" | "external" | "false" | "field" | "file"
        | "final" | "finally" | "for" | "fun" | "get" | "if" | "import" | "in" | "infix"
        | "init" | "inline" | "inner" | "interface" | "internal" | "is" | "it" | "lateinit"
        | "noinline" | "null" | "object" | "open" | "operator" | "out" | "override" | "package"
        | "param" | "private" | "property" | "protected" | "public" | "receiver" | "reified"
        | "return" | "sealed" | "set" | "setparam" | "super" | "suspend" | "tailrec" | "this"
        | "throw" | "true" | "try" | "typealias" | "typeof" | "val" | "value" | "var"
        | "vararg" | "when" | "where" | "while" => format!("`{normalized}`"),
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
        assert_eq!(Identifier::try_from("fun").unwrap().as_ref(), "`fun`");
        assert_eq!(
            Identifier::try_from("package").unwrap().as_ref(),
            "`package`"
        );
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
