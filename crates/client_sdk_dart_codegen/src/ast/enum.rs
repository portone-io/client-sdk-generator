use std::fmt;

use super::{Comment, Identifier, Indent, UnionParent};

#[derive(Debug, Clone)]
pub struct Enum {
    pub name: Identifier,
    pub description: Option<Comment>,
    pub variants: Vec<EnumVariant>,
    pub union_parents: Vec<UnionParent>,
}

#[derive(Debug, Clone)]
pub struct EnumVariant {
    pub name: Identifier,
    pub value: String,
    pub description: Option<Comment>,
}

impl fmt::Display for Enum {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for comment in self.description.iter().flat_map(Comment::lines) {
            writeln!(f, "/// {comment}")?;
        }
        writeln!(f, "enum {name} {{", name = self.name.as_ref())?;
        {
            let indent = Indent(1);
            let len = self.variants.len();
            for (i, variant) in self.variants.iter().enumerate() {
                let terminator = if i + 1 == len { ";" } else { "," };
                for comment in variant.description.iter().flat_map(Comment::lines) {
                    writeln!(f, "{indent}/// {comment}")?;
                }
                writeln!(f, "{indent}{variant}{terminator}")?;
            }
            writeln!(f)?;
            writeln!(f, "{indent}final String _value;")?;
            writeln!(
                f,
                "{indent}const {name}(String value) : _value = value;",
                name = self.name.as_ref()
            )?;
            writeln!(f, "{indent}String toJson() => _value;")?;
        }
        writeln!(f, "}}")
    }
}

impl fmt::Display for EnumVariant {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{name}('{value}')",
            name = self.name.as_ref(),
            value = self.value
        )?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn enum_with_variant() {
        let empty = Enum {
            name: Identifier::try_from("Test1").unwrap(),
            description: Some(Comment::try_from("Test1 Enum").unwrap()),
            variants: vec![
                EnumVariant {
                    name: Identifier::try_from("VARIANT_A").unwrap(),
                    value: "value_a".into(),
                    description: None,
                },
                EnumVariant {
                    name: Identifier::try_from("VARIANT_B").unwrap(),
                    value: "value_b".into(),
                    description: Some(
                        Comment::try_from("This is a variant\nwith a multi-line description")
                            .unwrap(),
                    ),
                },
            ],
            union_parents: vec![],
        };
        assert_eq!(
            empty.to_string(),
            r"/// Test1 Enum
enum Test1 {
    VARIANT_A('value_a'),
    /// This is a variant
    /// with a multi-line description
    VARIANT_B('value_b');

    final String _value;
    const Test1(String value) : _value = value;
    String toJson() => _value;
}
"
        );
    }
}
