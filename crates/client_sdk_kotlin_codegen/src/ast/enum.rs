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
        if let Some(ref desc) = self.description {
            writeln!(f, "/**")?;
            for line in desc.lines() {
                writeln!(f, " * {line}")?;
            }
            writeln!(f, " */")?;
        }
        writeln!(f, "enum class {name} {{", name = self.name.as_ref())?;
        {
            let indent = Indent(1);
            let len = self.variants.len();
            for (i, variant) in self.variants.iter().enumerate() {
                let terminator = if i + 1 == len { ";" } else { "," };
                if let Some(ref desc) = variant.description {
                    writeln!(f, "{indent}/**")?;
                    for line in desc.lines() {
                        writeln!(f, "{indent} * {line}")?;
                    }
                    writeln!(f, "{indent} */")?;
                }
                writeln!(f, "{indent}{variant}{terminator}")?;
            }
            writeln!(f)?;
            writeln!(f, "{indent}fun toJson(): String = name")?;
        }
        writeln!(f, "}}")
    }
}

impl fmt::Display for EnumVariant {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{name}", name = self.name.as_ref())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn enum_with_variant() {
        let enum_entity = Enum {
            name: Identifier::try_from("Bank").unwrap(),
            description: Some(
                Comment::try_from("계좌이체, 가상계좌 발급시 사용되는 은행 코드").unwrap(),
            ),
            variants: vec![
                EnumVariant {
                    name: Identifier::try_from("BANK_OF_KOREA").unwrap(),
                    value: "BANK_OF_KOREA".into(),
                    description: Some(Comment::try_from("한국은행").unwrap()),
                },
                EnumVariant {
                    name: Identifier::try_from("KOREA_DEVELOPMENT_BANK").unwrap(),
                    value: "KOREA_DEVELOPMENT_BANK".into(),
                    description: Some(Comment::try_from("산업은행").unwrap()),
                },
                EnumVariant {
                    name: Identifier::try_from("CAPE_INVESTMENT_CERTIFICATE").unwrap(),
                    value: "CAPE_INVESTMENT_CERTIFICATE".into(),
                    description: Some(Comment::try_from("케이프투자증권").unwrap()),
                },
            ],
            union_parents: vec![],
        };

        let expected = r#"/**
 * 계좌이체, 가상계좌 발급시 사용되는 은행 코드
 */
enum class Bank {
    /**
     * 한국은행
     */
    BANK_OF_KOREA,
    /**
     * 산업은행
     */
    KOREA_DEVELOPMENT_BANK,
    /**
     * 케이프투자증권
     */
    CAPE_INVESTMENT_CERTIFICATE;

    fun toJson(): String = name
}
"#;

        assert_eq!(enum_entity.to_string(), expected);
    }
}
