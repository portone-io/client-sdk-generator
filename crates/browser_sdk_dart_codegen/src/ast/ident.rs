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
        if value.is_ascii() {
            Ok(Identifier(value))
        } else {
            Err(format!("non-ascii identifier: {}", value))
        }
    }
}

impl TryFrom<&str> for Identifier {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        if value.is_ascii() {
            Ok(Identifier(value.into()))
        } else {
            Err(format!("non-ascii identifier: {}", value))
        }
    }
}
