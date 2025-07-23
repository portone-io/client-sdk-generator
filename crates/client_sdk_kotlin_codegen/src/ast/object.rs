use std::fmt;

use super::{Comment, CompositeType, Identifier, Indent, ScalarType, UnionParent};

#[derive(Debug, Clone)]
pub struct Object {
    pub name: Identifier,
    pub description: Option<Comment>,
    pub fields: Vec<ObjectField>,
    pub union_parents: Vec<UnionParent>,
    pub is_one_of: bool,
}

impl fmt::Display for Object {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(ref desc) = self.description {
            writeln!(f, "/**")?;
            for line in desc.lines() {
                writeln!(f, " * {line}")?;
            }
            writeln!(f, " */")?;
        }

        if self.fields.is_empty() && !self.is_one_of {
            // Empty object case
            writeln!(f, "class {name} {{", name = self.name.as_ref())?;
            {
                let indent = Indent(1);
                writeln!(f, "{indent}fun toJson(): Map<String, Any> = emptyMap()")?;
            }
            writeln!(f, "}}")
        } else if self.is_one_of {
            // OneOf (sealed interface) case
            writeln!(f, "sealed interface {name} {{", name = self.name.as_ref())?;
            {
                let indent = Indent(1);
                for field in self.fields.iter() {
                    if let Some(ref desc) = field.description {
                        let lines: Vec<&str> = desc.lines().collect();
                        writeln!(f, "{indent}/**")?;
                        for line in lines {
                            writeln!(f, "{indent} * {line}")?;
                        }
                        writeln!(f, "{indent} */")?;
                    }
                    let field_name_pascal = capitalize_first(field.name.as_ref());
                    let field_type = if field.value_type.is_list {
                        format!("List<{}>", field.value_type.scalar.to_identifier())
                    } else {
                        field.value_type.scalar.to_identifier().to_string()
                    };
                    writeln!(
                        f,
                        "{indent}data class {field_name_pascal}(val value: {field_type}) : {name}",
                        name = self.name.as_ref()
                    )?;
                }
                writeln!(f)?;
                writeln!(f, "{indent}fun toJson(): Map<String, Any> = when (this) {{")?;
                {
                    let indent = Indent(2);
                    for field in self.fields.iter() {
                        let field_name_pascal = capitalize_first(field.name.as_ref());
                        let to_json = ToJson {
                            name: "value",
                            is_list: field.value_type.is_list,
                            scalar: &field.value_type.scalar,
                        };
                        writeln!(
                            f,
                            "{indent}is {field_name_pascal} -> mapOf(\"{serialized_name}\" to {to_json})",
                            serialized_name = field.serialized_name
                        )?;
                    }
                }
                writeln!(f, "{indent}}}")?;
            }
            writeln!(f, "}}")
        } else {
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
            writeln!(f, ") {{")?;
            {
                let indent = Indent(1);
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
}

#[derive(Debug, Clone)]
pub struct ObjectField {
    pub name: Identifier,
    pub serialized_name: String,
    pub value_type: CompositeType,
    pub description: Option<Comment>,
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
            ScalarType::Int
            | ScalarType::Double
            | ScalarType::Boolean
            | ScalarType::Json
            | ScalarType::String => {
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

impl fmt::Display for ObjectField {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let nullable = if self.value_type.is_required { "" } else { "?" };
        let field_type = if self.value_type.is_list {
            format!("List<{}>", self.value_type.scalar.to_identifier())
        } else {
            self.value_type.scalar.to_identifier().to_string()
        };
        write!(
            f,
            "val {name}: {field_type}{nullable}",
            name = self.name.as_ref()
        )
    }
}

fn capitalize_first(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
    }
}

#[cfg(test)]
mod tests {
    use crate::ast::{ScalarType, TypeReference};

    use super::*;

    #[test]
    fn empty_object() {
        let object = Object {
            name: Identifier::try_from("IssueBillingKeyRequestUnionPaypal").unwrap(),
            description: None,
            fields: vec![],
            is_one_of: false,
            union_parents: vec![],
        };
        assert_eq!(
            object.to_string(),
            r#"class IssueBillingKeyRequestUnionPaypal {
    fun toJson(): Map<String, Any> = emptyMap()
}
"#
        );
    }

    #[test]
    fn object_with_fields() {
        let object = Object {
            name: Identifier::try_from("Address").unwrap(),
            description: Some(Comment::try_from("주소 정보").unwrap()),
            fields: vec![
                ObjectField {
                    name: Identifier::try_from("country").unwrap(),
                    serialized_name: "country".to_string(),
                    value_type: CompositeType {
                        scalar: ScalarType::TypeReference(TypeReference {
                            name: Identifier::try_from("Country").unwrap(),
                            path: "".into(),
                        }),
                        is_list: false,
                        is_required: false,
                    },
                    description: None,
                },
                ObjectField {
                    name: Identifier::try_from("addressLine1").unwrap(),
                    serialized_name: "addressLine1".to_string(),
                    value_type: CompositeType {
                        scalar: ScalarType::String,
                        is_list: false,
                        is_required: true,
                    },
                    description: Some(Comment::try_from("**일반주소**").unwrap()),
                },
                ObjectField {
                    name: Identifier::try_from("addressLine2").unwrap(),
                    serialized_name: "addressLine2".to_string(),
                    value_type: CompositeType {
                        scalar: ScalarType::String,
                        is_list: false,
                        is_required: true,
                    },
                    description: Some(Comment::try_from("**상세주소**").unwrap()),
                },
                ObjectField {
                    name: Identifier::try_from("city").unwrap(),
                    serialized_name: "city".to_string(),
                    value_type: CompositeType {
                        scalar: ScalarType::String,
                        is_list: false,
                        is_required: false,
                    },
                    description: Some(Comment::try_from("**도시**").unwrap()),
                },
                ObjectField {
                    name: Identifier::try_from("province").unwrap(),
                    serialized_name: "province".to_string(),
                    value_type: CompositeType {
                        scalar: ScalarType::String,
                        is_list: false,
                        is_required: false,
                    },
                    description: Some(Comment::try_from("**주, 도, 시**").unwrap()),
                },
            ],
            is_one_of: false,
            union_parents: vec![],
        };
        assert_eq!(
            object.to_string(),
            r#"/**
 * 주소 정보
 */
data class Address(
    val country: Country?,
    /**
     * **일반주소**
     */
    val addressLine1: String,
    /**
     * **상세주소**
     */
    val addressLine2: String,
    /**
     * **도시**
     */
    val city: String?,
    /**
     * **주, 도, 시**
     */
    val province: String?
) {
    fun toJson(): Map<String, Any?> = mapOf(
        "country" to country?.let { country.toJson() },
        "addressLine1" to addressLine1,
        "addressLine2" to addressLine2,
        "city" to city?.let { city },
        "province" to province?.let { province }
    )
}
"#
        );
    }

    #[test]
    fn one_of_object() {
        let object = Object {
            name: Identifier::try_from("MonthOption").unwrap(),
            description: Some(Comment::try_from("**할부 개월 수 설정**").unwrap()),
            fields: vec![
                ObjectField {
                    name: Identifier::try_from("fixedMonth").unwrap(),
                    serialized_name: "fixedMonth".to_string(),
                    value_type: CompositeType {
                        scalar: ScalarType::Int,
                        is_list: false,
                        is_required: true,
                    },
                    description: Some(
                        Comment::try_from("**구매자가 선택할 수 없도록 고정된 할부 개월수**")
                            .unwrap(),
                    ),
                },
                ObjectField {
                    name: Identifier::try_from("availableMonthList").unwrap(),
                    serialized_name: "availableMonthList".to_string(),
                    value_type: CompositeType {
                        scalar: ScalarType::Int,
                        is_list: true,
                        is_required: true,
                    },
                    description: Some(
                        Comment::try_from("**구매자가 선택할 수 있는 할부 개월수 리스트**")
                            .unwrap(),
                    ),
                },
            ],
            is_one_of: true,
            union_parents: vec![],
        };
        assert_eq!(
            object.to_string(),
            r#"/**
 * **할부 개월 수 설정**
 */
sealed interface MonthOption {
    /**
     * **구매자가 선택할 수 없도록 고정된 할부 개월수**
     */
    data class FixedMonth(val value: Int) : MonthOption
    /**
     * **구매자가 선택할 수 있는 할부 개월수 리스트**
     */
    data class AvailableMonthList(val value: List<Int>) : MonthOption

    fun toJson(): Map<String, Any> = when (this) {
        is FixedMonth -> mapOf("fixedMonth" to value)
        is AvailableMonthList -> mapOf("availableMonthList" to value)
    }
}
"#
        );
    }
}
