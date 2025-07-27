use std::fmt;

use super::{Comment, Identifier, Indent, ObjectField, ScalarType, TypeReference, UnionParent};

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
            writeln!(f, "/**")?;
            for line in desc.lines() {
                writeln!(f, " * {line}")?;
            }
            writeln!(f, " */")?;
        }

        // Data class declaration with flattened fields
        writeln!(f, "@Parcelize")?;
        writeln!(f, "data class {name}(", name = self.name.as_ref())?;
        {
            let indent = Indent(1);
            for (i, field) in self.fields.iter().enumerate() {
                let terminator = if i + 1 == self.fields.len() { "" } else { "," };
                if let Some(ref desc) = field.description {
                    writeln!(f, "{indent}/**")?;
                    for line in desc.lines() {
                        writeln!(f, "{indent} * {line}")?;
                    }
                    writeln!(f, "{indent} */")?;
                }
                writeln!(f, "{indent}{field}{terminator}")?;
            }
        }
        writeln!(f, ") : Parcelable {{")?;

        {
            let indent = Indent(1);

            // toJson method with flattened fields
            writeln!(f, "{indent}fun toJson(): Map<String, Any?> = mapOf(")?;
            {
                let indent = Indent(2);
                for (i, field) in self.fields.iter().enumerate() {
                    let terminator = if i + 1 == self.fields.len() { "" } else { "," };
                    let to_json = ToJson {
                        name: field.name.as_ref(),
                        is_list: field.value_type.is_list,
                        scalar: &field.value_type.scalar,
                    };

                    if field.value_type.is_required {
                        writeln!(
                            f,
                            "{indent}\"{serialized_name}\" to {to_json}{terminator}",
                            serialized_name = field.serialized_name
                        )?;
                    } else {
                        writeln!(
                            f,
                            "{indent}\"{serialized_name}\" to {field_name}?.let {{ {to_json} }}{terminator}",
                            serialized_name = field.serialized_name,
                            field_name = field.name.as_ref()
                        )?;
                    }
                }
            }
            writeln!(f, "{indent})")?;
        }

        writeln!(f, "}}")
    }
}

struct ToJson<'a> {
    name: &'a str,
    is_list: bool,
    scalar: &'a ScalarType,
}

impl fmt::Display for ToJson<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = self.name;
        match self.scalar {
            ScalarType::Long | ScalarType::Boolean | ScalarType::Json | ScalarType::String => {
                write!(f, "{name}")
            }
            ScalarType::TypeReference(_) => {
                if self.is_list {
                    write!(f, "{name}.map {{ it.toJson() }}")
                } else {
                    write!(f, "{name}.toJson()")
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::ast::CompositeType;

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
                        scalar: ScalarType::Long,
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
            r#"/**
 * 결제 요청 정보
 */
@Parcelize
data class PaymentRequest(
    /**
     * 결제 금액
     */
    val amount: Long,
    /**
     * 통화 코드
     */
    val currency: String,
    /**
     * 결제 수단
     */
    val method: PaymentMethod,
    /**
     * 카드 정보
     */
    val cardInfo: CardInfo?
) : Parcelable {
    fun toJson(): Map<String, Any?> = mapOf(
        "amount" to amount,
        "currency" to currency,
        "method" to method.toJson(),
        "cardInfo" to cardInfo?.let { cardInfo.toJson() }
    )
}
"#
        );
    }
}
