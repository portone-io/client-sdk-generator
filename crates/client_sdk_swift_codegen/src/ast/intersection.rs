use std::fmt;

use super::{Comment, Identifier, Indent, ObjectField, TypeReference, UnionParent};

#[derive(Debug, Clone)]
pub struct Intersection {
    pub name: Identifier,
    pub description: Option<Comment>,
    pub constituents: Vec<IntersectionConstituent>,
    pub fields: Vec<ObjectField>, // Flattened fields from all constituents
    pub union_parents: Vec<UnionParent>,
}

#[derive(Debug, Clone)]
pub struct IntersectionConstituent {
    pub name: Identifier,
    pub type_name: TypeReference,
}

impl fmt::Display for Intersection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Comments
        if let Some(ref desc) = self.description {
            for line in desc.lines() {
                writeln!(f, "/// {line}")?;
            }
        }

        // Struct declaration with flattened fields
        writeln!(
            f,
            "public struct {name}: Codable {{",
            name = self.name.as_ref()
        )?;
        {
            let indent = Indent(1);

            // Properties
            for field in self.fields.iter() {
                if let Some(ref desc) = field.description {
                    for line in desc.lines() {
                        writeln!(f, "{indent}/// {line}")?;
                    }
                }
                writeln!(f, "{indent}{field}")?;
            }
            writeln!(f)?;

            // CodingKeys
            let needs_coding_keys = self
                .fields
                .iter()
                .any(|field| field.name.as_ref() != field.serialized_name);
            if needs_coding_keys {
                writeln!(f, "{indent}private enum CodingKeys: String, CodingKey {{")?;
                {
                    let indent = Indent(2);
                    for field in self.fields.iter() {
                        let name = field.name.as_ref();
                        let serialized = &field.serialized_name;
                        if name == serialized {
                            writeln!(f, "{indent}case {name}")?;
                        } else {
                            writeln!(f, "{indent}case {name} = \"{serialized}\"")?;
                        }
                    }
                }
                writeln!(f, "{indent}}}")?;
                writeln!(f)?;
            }

            // Public initializer
            write!(f, "{indent}public init(")?;
            for (i, field) in self.fields.iter().enumerate() {
                let separator = if i > 0 { ", " } else { "" };
                let nullable = if field.value_type.is_required {
                    ""
                } else {
                    "?"
                };
                let default_value = if field.value_type.is_required {
                    ""
                } else {
                    " = nil"
                };
                let field_type = if field.value_type.is_list {
                    format!("[{}]", field.value_type.scalar.to_swift_type())
                } else {
                    field.value_type.scalar.to_swift_type().to_string()
                };
                write!(
                    f,
                    "{separator}{field_name}: {field_type}{nullable}{default_value}",
                    field_name = field.name.as_ref()
                )?;
            }
            writeln!(f, ") {{")?;
            {
                let indent = Indent(2);
                for field in self.fields.iter() {
                    let field_name = field.name.as_ref();
                    writeln!(f, "{indent}self.{field_name} = {field_name}")?;
                }
            }
            writeln!(f, "{indent}}}")?;
        }
        writeln!(f, "}}")
    }
}

#[cfg(test)]
mod tests {
    use crate::ast::{CompositeType, ScalarType};

    use super::*;

    #[test]
    fn intersection_type() {
        let intersection = Intersection {
            name: Identifier::try_from("PaymentRequest").unwrap(),
            description: Some(Comment::try_from("결제 요청 정보").unwrap()),
            constituents: vec![
                IntersectionConstituent {
                    name: Identifier::try_from("paymentRequestBase").unwrap(),
                    type_name: TypeReference {
                        name: Identifier::try_from("PaymentRequestBase").unwrap(),
                        path: "request/payment_request_base".into(),
                    },
                },
                IntersectionConstituent {
                    name: Identifier::try_from("paymentRequestUnion").unwrap(),
                    type_name: TypeReference {
                        name: Identifier::try_from("PaymentRequestUnion").unwrap(),
                        path: "request/payment_request_union".into(),
                    },
                },
            ],
            fields: vec![
                // Fields from PaymentRequestBase
                ObjectField {
                    name: Identifier::try_from("amount").unwrap(),
                    serialized_name: "amount".to_string(),
                    value_type: CompositeType {
                        scalar: ScalarType::Int,
                        is_list: false,
                        is_required: true,
                    },
                    description: Some(Comment::try_from("결제 금액").unwrap()),
                },
                ObjectField {
                    name: Identifier::try_from("currency").unwrap(),
                    serialized_name: "currency".to_string(),
                    value_type: CompositeType {
                        scalar: ScalarType::String,
                        is_list: false,
                        is_required: true,
                    },
                    description: Some(Comment::try_from("통화 코드").unwrap()),
                },
                // Fields from PaymentRequestUnion
                ObjectField {
                    name: Identifier::try_from("method").unwrap(),
                    serialized_name: "method".to_string(),
                    value_type: CompositeType {
                        scalar: ScalarType::TypeReference(TypeReference {
                            name: Identifier::try_from("PaymentMethod").unwrap(),
                            path: "entity/payment_method".into(),
                        }),
                        is_list: false,
                        is_required: true,
                    },
                    description: Some(Comment::try_from("결제 수단").unwrap()),
                },
                ObjectField {
                    name: Identifier::try_from("cardInfo").unwrap(),
                    serialized_name: "cardInfo".to_string(),
                    value_type: CompositeType {
                        scalar: ScalarType::TypeReference(TypeReference {
                            name: Identifier::try_from("CardInfo").unwrap(),
                            path: "entity/card_info".into(),
                        }),
                        is_list: false,
                        is_required: false,
                    },
                    description: Some(Comment::try_from("카드 정보").unwrap()),
                },
            ],
            union_parents: vec![],
        };

        assert_eq!(
            intersection.to_string(),
            r#"/// 결제 요청 정보
public struct PaymentRequest: Codable {
    /// 결제 금액
    public let amount: Int
    /// 통화 코드
    public let currency: String
    /// 결제 수단
    public let method: PaymentMethod
    /// 카드 정보
    public let cardInfo: CardInfo?

    public init(amount: Int, currency: String, method: PaymentMethod, cardInfo: CardInfo? = nil) {
        self.amount = amount
        self.currency = currency
        self.method = method
        self.cardInfo = cardInfo
    }
}
"#
        );
    }
}
