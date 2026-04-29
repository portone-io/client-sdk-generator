use std::fmt;

use crate::ast::Indent;

use super::{Comment, Identifier, TypeReference};

#[derive(Debug, Clone)]
pub enum UnionParent {
    Union {
        parent: TypeReference,
        variant_name: Identifier,
    },
}

#[derive(Debug, Clone)]
pub struct Union {
    pub name: Identifier,
    pub description: Option<Comment>,
    pub variants: Vec<UnionVariant>,
}

#[derive(Debug, Clone)]
pub struct UnionVariant {
    pub name: Identifier,
    pub type_name: TypeReference,
    pub description: Option<Comment>,
}

impl fmt::Display for Union {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Comments
        if let Some(ref desc) = self.description {
            for line in desc.lines() {
                writeln!(f, "/// {line}")?;
            }
        }

        // Enum declaration with associated values
        writeln!(f, "public enum {name}: Codable {{", name = self.name.as_ref())?;
        {
            let indent = Indent(1);

            // Variant declarations
            for variant in self.variants.iter() {
                if let Some(ref desc) = variant.description {
                    for line in desc.lines() {
                        writeln!(f, "{indent}/// {line}")?;
                    }
                }
                writeln!(
                    f,
                    "{indent}case {variant_name}({type_name})",
                    variant_name = decapitalize_first(variant.name.as_ref()),
                    type_name = variant.type_name.name.as_ref(),
                )?;
            }

            writeln!(f)?;

            // Custom Decodable init
            writeln!(f, "{indent}public init(from decoder: Decoder) throws {{")?;
            {
                let indent = Indent(2);
                for (i, variant) in self.variants.iter().enumerate() {
                    let variant_name = decapitalize_first(variant.name.as_ref());
                    let type_name = variant.type_name.name.as_ref();
                    if i == 0 {
                        writeln!(f, "{indent}if let value = try? {type_name}(from: decoder) {{")?;
                    } else {
                        writeln!(f, "{indent}}} else if let value = try? {type_name}(from: decoder) {{")?;
                    }
                    {
                        let indent = Indent(3);
                        writeln!(f, "{indent}self = .{variant_name}(value)")?;
                    }
                }
                writeln!(f, "{indent}}} else {{")?;
                {
                    let indent = Indent(3);
                    writeln!(f, "{indent}throw DecodingError.dataCorrupted(DecodingError.Context(codingPath: decoder.codingPath, debugDescription: \"No matching type found\"))")?;
                }
                writeln!(f, "{indent}}}")?;
            }
            writeln!(f, "{indent}}}")?;
            writeln!(f)?;

            // Custom Encodable encode
            writeln!(f, "{indent}public func encode(to encoder: Encoder) throws {{")?;
            {
                let indent = Indent(2);
                writeln!(f, "{indent}switch self {{")?;
                for variant in self.variants.iter() {
                    let variant_name = decapitalize_first(variant.name.as_ref());
                    writeln!(f, "{indent}case .{variant_name}(let value):")?;
                    {
                        let indent = Indent(3);
                        writeln!(f, "{indent}try value.encode(to: encoder)")?;
                    }
                }
                writeln!(f, "{indent}}}")?;
            }
            writeln!(f, "{indent}}}")?;
        }
        writeln!(f, "}}")
    }
}

fn decapitalize_first(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_lowercase().collect::<String>() + chars.as_str(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn union_with_variant() {
        let union = Union {
            name: Identifier::try_from("LoadableUIType").unwrap(),
            description: None,
            variants: vec![
                UnionVariant {
                    name: Identifier::try_from("PaymentUIType").unwrap(),
                    type_name: TypeReference {
                        name: Identifier::try_from("PaymentUIType").unwrap(),
                        path: "".into(),
                    },
                    description: None,
                },
                UnionVariant {
                    name: Identifier::try_from("IssueBillingKeyUIType").unwrap(),
                    type_name: TypeReference {
                        name: Identifier::try_from("IssueBillingKeyUIType").unwrap(),
                        path: "".into(),
                    },
                    description: None,
                },
            ],
        };
        assert_eq!(
            union.to_string(),
            r#"public enum LoadableUIType: Codable {
    case paymentUIType(PaymentUIType)
    case issueBillingKeyUIType(IssueBillingKeyUIType)

    public init(from decoder: Decoder) throws {
        if let value = try? PaymentUIType(from: decoder) {
            self = .paymentUIType(value)
        } else if let value = try? IssueBillingKeyUIType(from: decoder) {
            self = .issueBillingKeyUIType(value)
        } else {
            throw DecodingError.dataCorrupted(DecodingError.Context(codingPath: decoder.codingPath, debugDescription: "No matching type found"))
        }
    }

    public func encode(to encoder: Encoder) throws {
        switch self {
        case .paymentUIType(let value):
            try value.encode(to: encoder)
        case .issueBillingKeyUIType(let value):
            try value.encode(to: encoder)
        }
    }
}
"#
        );
    }
}
