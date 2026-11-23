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
    pub match_json_shape: bool,
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
        writeln!(
            f,
            "public enum {name}: Codable {{",
            name = self.name.as_ref()
        )?;
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
                if self.match_json_shape {
                    writeln!(f, "{indent}let shape = try JSONShape(from: decoder)")?;
                }
                for (i, variant) in self.variants.iter().enumerate() {
                    let variant_name = decapitalize_first(variant.name.as_ref());
                    let type_name = variant.type_name.name.as_ref();
                    let shape_check = if self.match_json_shape {
                        ", (try? JSONShape.of(value)) == shape"
                    } else {
                        ""
                    };
                    if i == 0 {
                        writeln!(
                            f,
                            "{indent}if let value = try? {type_name}(from: decoder){shape_check} {{"
                        )?;
                    } else {
                        writeln!(
                            f,
                            "{indent}}} else if let value = try? {type_name}(from: decoder){shape_check} {{"
                        )?;
                    }
                    {
                        let indent = Indent(3);
                        writeln!(f, "{indent}self = .{variant_name}(value)")?;
                    }
                }
                writeln!(f, "{indent}}} else {{")?;
                {
                    let indent = Indent(3);
                    writeln!(
                        f,
                        "{indent}throw DecodingError.dataCorrupted(DecodingError.Context(codingPath: decoder.codingPath, debugDescription: \"No matching type found\"))"
                    )?;
                }
                writeln!(f, "{indent}}}")?;
            }
            writeln!(f, "{indent}}}")?;
            writeln!(f)?;

            // Custom Encodable encode
            writeln!(
                f,
                "{indent}public func encode(to encoder: Encoder) throws {{"
            )?;
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
        if self.match_json_shape {
            write!(f, "{JSON_SHAPE}")?;
        }
        writeln!(f, "}}")
    }
}

// Swift's synthesized Decodable accepts unknown keys. Compare nested key/array
// shapes as well as successfully decoding the candidate to avoid losing fields.
const JSON_SHAPE: &str = r#"
    private enum JSONShape: Decodable, Equatable {
        case object([String: JSONShape])
        case array([JSONShape])
        case scalar
        case null

        private struct Key: CodingKey {
            let stringValue: String
            let intValue: Int? = nil

            init(stringValue: String) { self.stringValue = stringValue }
            init?(intValue: Int) { return nil }
        }

        init(from decoder: Decoder) throws {
            if let container = try? decoder.container(keyedBy: Key.self) {
                var fields: [String: JSONShape] = [:]
                for key in container.allKeys {
                    fields[key.stringValue] = try container.decode(JSONShape.self, forKey: key)
                }
                self = .object(fields)
            } else if var container = try? decoder.unkeyedContainer() {
                var elements: [JSONShape] = []
                while !container.isAtEnd {
                    elements.append(try container.decode(JSONShape.self))
                }
                self = .array(elements)
            } else {
                self = try decoder.singleValueContainer().decodeNil() ? .null : .scalar
            }
        }

        static func of<Value: Encodable>(_ value: Value) throws -> JSONShape {
            try JSONDecoder().decode(JSONShape.self, from: JSONEncoder().encode(value))
        }
    }
"#;

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
            match_json_shape: false,
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
