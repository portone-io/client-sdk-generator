#[derive(Debug, Clone)]
pub struct Identifier(String);

impl AsRef<str> for Identifier {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for Identifier {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::try_from(value.as_str())
    }
}

impl TryFrom<&str> for Identifier {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let normalized = normalize_identifier(value);
        if normalized.is_empty() {
            Err(format!("empty identifier: {value}"))
        } else {
            Ok(Identifier(normalized))
        }
    }
}

fn normalize_identifier(s: &str) -> String {
    let mut normalized = s.replace(['-', ' ', '.', '/'], "_");

    if normalized.chars().next().is_some_and(|c| c.is_numeric()) {
        normalized = format!("_{normalized}");
    }

    // Handle Dart reserved keywords
    match normalized.as_str() {
        "abstract" | "as" | "assert" | "async" | "await" | "break" | "case" | "catch" | "class"
        | "const" | "continue" | "covariant" | "default" | "deferred" | "do" | "dynamic"
        | "else" | "enum" | "export" | "extends" | "extension" | "external" | "factory"
        | "false" | "final" | "finally" | "for" | "Function" | "get" | "hide" | "if"
        | "implements" | "import" | "in" | "interface" | "is" | "late" | "library" | "mixin"
        | "new" | "null" | "of" | "on" | "operator" | "part" | "required" | "rethrow"
        | "return" | "sealed" | "set" | "show" | "static" | "super" | "switch" | "sync"
        | "this" | "throw" | "true" | "try" | "typedef" | "var" | "void" | "when" | "while"
        | "with" | "yield" => format!("${normalized}"),
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
        assert_eq!(Identifier::try_from("class").unwrap().as_ref(), "$class");
        assert_eq!(Identifier::try_from("final").unwrap().as_ref(), "$final");
        assert_eq!(Identifier::try_from("sealed").unwrap().as_ref(), "$sealed");
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
