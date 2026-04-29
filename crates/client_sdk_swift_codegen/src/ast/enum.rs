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
            writeln!(f, "/// {}", desc.lines().collect::<Vec<_>>().join("\n/// "))?;
        }
        writeln!(
            f,
            "public enum {name}: String, Codable {{",
            name = self.name.as_ref()
        )?;
        {
            let indent = Indent(1);
            for variant in self.variants.iter() {
                if let Some(ref desc) = variant.description {
                    for line in desc.lines() {
                        writeln!(f, "{indent}/// {line}")?;
                    }
                }
                writeln!(f, "{indent}{variant}")?;
            }
        }
        writeln!(f, "}}")
    }
}

impl fmt::Display for EnumVariant {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = self.name.as_ref();
        let value = &self.value;
        if name == value {
            write!(f, "case {name}")
        } else {
            write!(f, "case {name} = \"{value}\"")
        }
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

        let expected = r#"/// 계좌이체, 가상계좌 발급시 사용되는 은행 코드
public enum Bank: String, Codable {
    /// 한국은행
    case BANK_OF_KOREA
    /// 산업은행
    case KOREA_DEVELOPMENT_BANK
    /// 케이프투자증권
    case CAPE_INVESTMENT_CERTIFICATE
}
"#;

        assert_eq!(enum_entity.to_string(), expected);
    }

    #[test]
    fn enum_with_numeric_variant() {
        let enum_entity = Enum {
            name: Identifier::try_from("PaymentMethod").unwrap(),
            description: Some(Comment::try_from("결제 수단").unwrap()),
            variants: vec![
                EnumVariant {
                    name: Identifier::try_from("2checkout").unwrap(),
                    value: "2checkout".into(),
                    description: Some(Comment::try_from("2Checkout 결제").unwrap()),
                },
                EnumVariant {
                    name: Identifier::try_from("3ds").unwrap(),
                    value: "3ds".into(),
                    description: Some(Comment::try_from("3D Secure 인증").unwrap()),
                },
                EnumVariant {
                    name: Identifier::try_from("card").unwrap(),
                    value: "card".into(),
                    description: Some(Comment::try_from("카드 결제").unwrap()),
                },
            ],
            union_parents: vec![],
        };

        let expected = r#"/// 결제 수단
public enum PaymentMethod: String, Codable {
    /// 2Checkout 결제
    case _2checkout = "2checkout"
    /// 3D Secure 인증
    case _3ds = "3ds"
    /// 카드 결제
    case card
}
"#;

        assert_eq!(enum_entity.to_string(), expected);
    }
}
