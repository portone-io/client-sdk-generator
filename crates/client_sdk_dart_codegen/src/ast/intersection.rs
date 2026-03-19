use core::fmt;

use super::{
    capitalize_first, Comment, Identifier, Indent, ObjectField, ScalarType, TypeReference,
    UnionParent,
};

pub struct Intersection {
    pub name: Identifier,
    pub description: Option<Comment>,
    pub constituents: Vec<IntersectionConstituent>,
    pub fields: Vec<ObjectField>,
    pub union_parents: Vec<UnionParent>,
}

pub struct IntersectionConstituent {
    pub name: Identifier,
    pub type_name: TypeReference,
}

impl fmt::Display for Intersection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for comment in self.description.iter().flat_map(Comment::lines) {
            writeln!(f, "/// {comment}")?;
        }
        writeln!(f, "class {name} {{", name = self.name.as_ref())?;
        {
            let indent = Indent(1);
            if self.fields.is_empty() {
                writeln!(f, "{indent}Map<String, dynamic> toJson() => {{}};")?;
                writeln!(
                    f,
                    "{indent}static {name} fromJson(Map<String, dynamic> json) => {name}();",
                    name = self.name.as_ref()
                )?;
            } else {
                for field in self.fields.iter() {
                    for comment in field.description.iter().flat_map(Comment::lines) {
                        writeln!(f, "{indent}/// {comment}")?;
                    }
                    writeln!(f, "{indent}final {field};")?;
                }
                writeln!(f)?;
                writeln!(f, "{indent}{name}({{", name = self.name.as_ref())?;
                {
                    let indent = Indent(2);
                    for field in self.fields.iter() {
                        if field.value_type.is_required {
                            writeln!(
                                f,
                                "{indent}required this.{field_name},",
                                field_name = field.name.as_ref()
                            )?;
                        } else {
                            writeln!(
                                f,
                                "{indent}this.{field_name},",
                                field_name = field.name.as_ref()
                            )?;
                        }
                    }
                }
                writeln!(f, "{indent}}});")?;
                writeln!(f)?;
                writeln!(f, "{indent}Map<String, dynamic> toJson() => {{")?;
                {
                    let indent = Indent(2);
                    for field in self.fields.iter() {
                        let to_json = ToJson {
                            name: field.name.as_ref(),
                            is_list: field.value_type.is_list,
                            scalar: &field.value_type.scalar,
                            assert_non_null: !field.value_type.is_required,
                            null_aware_call: false,
                        };
                        if to_json.assert_non_null {
                            writeln!(
                                f,
                                "{indent}if ({field_name} != null) '{serialized_name}': {to_json},",
                                field_name = field.name.as_ref(),
                                serialized_name = field.serialized_name,
                            )?;
                        } else {
                            writeln!(
                                f,
                                "{indent}'{serialized_name}': {to_json},",
                                serialized_name = field.serialized_name,
                            )?;
                        }
                    }
                }
                writeln!(f, "{indent}}};")?;
                writeln!(f)?;
                writeln!(
                    f,
                    "{indent}static {name} fromJson(Map<String, dynamic> json) => {name}(",
                    name = self.name.as_ref()
                )?;
                {
                    let indent = Indent(2);
                    for field in self.fields.iter() {
                        let from_json = FromJson {
                            serialized_name: &field.serialized_name,
                            is_list: field.value_type.is_list,
                            scalar: &field.value_type.scalar,
                            is_required: field.value_type.is_required,
                        };
                        writeln!(
                            f,
                            "{indent}{field_name}: {from_json},",
                            field_name = field.name.as_ref(),
                        )?;
                    }
                }
                writeln!(f, "{indent});")?;
            }
            if !self.union_parents.is_empty() {
                writeln!(f)?;
                for parent in self.union_parents.iter() {
                    match parent {
                        UnionParent::Union {
                            parent,
                            variant_name,
                        } => {
                            writeln!(
                                f,
                                "{indent}{parent_name} to{parent_name}() => {parent_name}{variant_pascal}(this);",
                                parent_name = parent.name.as_ref(),
                                variant_pascal = capitalize_first(variant_name.as_ref()),
                            )?;
                        }
                    }
                }
            }
        }
        writeln!(f, "}}")
    }
}

struct ToJson<'a> {
    name: &'a str,
    is_list: bool,
    scalar: &'a ScalarType,
    assert_non_null: bool,
    null_aware_call: bool,
}

impl fmt::Display for ToJson<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = self.name;
        let non_null = if self.assert_non_null { "!" } else { "" };
        match self.scalar {
            ScalarType::Int
            | ScalarType::Double
            | ScalarType::Bool
            | ScalarType::Object
            | ScalarType::String => {
                write!(f, "{name}{non_null}")
            }
            ScalarType::TypeReference(_) => {
                let call = if self.null_aware_call { "?" } else { "" };
                if self.is_list {
                    write!(f, "{name}{non_null}{call}.map((e) => e.toJson()).toList()")
                } else {
                    write!(f, "{name}{non_null}{call}.toJson()")
                }
            }
        }
    }
}

struct FromJson<'a> {
    serialized_name: &'a str,
    is_list: bool,
    scalar: &'a ScalarType,
    is_required: bool,
}

impl fmt::Display for FromJson<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let key = self.serialized_name;
        if self.is_list {
            match self.scalar {
                ScalarType::TypeReference(type_ref) => {
                    let type_name = type_ref.name.as_ref();
                    if self.is_required {
                        write!(f, "(json['{key}'] as List).map((e) => {type_name}.fromJson(e)).toList()")
                    } else {
                        write!(f, "(json['{key}'] as List?)?.map((e) => {type_name}.fromJson(e)).toList()")
                    }
                }
                scalar => {
                    let dart_type = scalar.to_identifier();
                    if self.is_required {
                        write!(f, "(json['{key}'] as List).cast<{dart_type}>()")
                    } else {
                        write!(f, "(json['{key}'] as List?)?.cast<{dart_type}>()")
                    }
                }
            }
        } else {
            match self.scalar {
                ScalarType::Double => {
                    if self.is_required {
                        write!(f, "(json['{key}'] as num).toDouble()")
                    } else {
                        write!(f, "(json['{key}'] as num?)?.toDouble()")
                    }
                }
                ScalarType::Object => {
                    write!(f, "json['{key}']")
                }
                ScalarType::TypeReference(type_ref) => {
                    let type_name = type_ref.name.as_ref();
                    if self.is_required {
                        write!(f, "{type_name}.fromJson(json['{key}'])")
                    } else {
                        write!(f, "json['{key}'] != null ? {type_name}.fromJson(json['{key}']) : null")
                    }
                }
                scalar => {
                    let dart_type = scalar.to_identifier();
                    let nullable = if self.is_required { "" } else { "?" };
                    write!(f, "json['{key}'] as {dart_type}{nullable}")
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
    fn test_intersection() {
        let intersection = Intersection {
            name: Identifier::try_from("PaymentRequest").unwrap(),
            description: Some(Comment::try_from("결제 요청 정보").unwrap()),
            constituents: vec![
                IntersectionConstituent {
                    name: Identifier::try_from("paymentRequestBase").unwrap(),
                    type_name: TypeReference {
                        name: Identifier::try_from("PaymentRequestBase").unwrap(),
                        path: "".into(),
                    },
                },
                IntersectionConstituent {
                    name: Identifier::try_from("paymentRequestUnion").unwrap(),
                    type_name: TypeReference {
                        name: Identifier::try_from("PaymentRequestUnion").unwrap(),
                        path: "".into(),
                    },
                },
            ],
            fields: vec![
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
            r"/// 결제 요청 정보
class PaymentRequest {
    /// 결제 금액
    final int amount;
    /// 통화 코드
    final String currency;
    /// 결제 수단
    final PaymentMethod method;
    /// 카드 정보
    final CardInfo? cardInfo;

    PaymentRequest({
        required this.amount,
        required this.currency,
        required this.method,
        this.cardInfo,
    });

    Map<String, dynamic> toJson() => {
        'amount': amount,
        'currency': currency,
        'method': method.toJson(),
        if (cardInfo != null) 'cardInfo': cardInfo!.toJson(),
    };

    static PaymentRequest fromJson(Map<String, dynamic> json) => PaymentRequest(
        amount: json['amount'] as int,
        currency: json['currency'] as String,
        method: PaymentMethod.fromJson(json['method']),
        cardInfo: json['cardInfo'] != null ? CardInfo.fromJson(json['cardInfo']) : null,
    );
}
"
        );
    }
}
