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
        for comment in self.description.iter().flat_map(Comment::lines) {
            writeln!(f, "/// {comment}")?;
        }
        writeln!(f, "class {name} {{", name = self.name.as_ref())?;
        {
            let indent = Indent(1);
            if self.fields.is_empty() {
                writeln!(f, "{indent}Map<String, dynamic> toJson() => {{}};")?;
            } else {
                for field in self.fields.iter() {
                    for comment in field.description.iter().flat_map(Comment::lines) {
                        writeln!(f, "{indent}/// {comment}")?;
                    }
                    if self.is_one_of {
                        let mut field = field.clone();
                        field.value_type.is_required = false;
                        writeln!(f, "{indent}final {field};")?;
                    } else {
                        writeln!(f, "{indent}final {field};")?;
                    }
                }
                writeln!(f)?;
                if self.is_one_of {
                    writeln!(f, "{indent}{name}.internal({{", name = self.name.as_ref())?;
                    {
                        let indent = Indent(2);
                        for field in self.fields.iter() {
                            writeln!(
                                f,
                                "{indent}this.{field_name},",
                                field_name = field.name.as_ref()
                            )?;
                        }
                    }
                    writeln!(f, "{indent}}});")?;
                    for field in self.fields.iter() {
                        writeln!(
                            f,
                            "{indent}{name}.{field_name}({field}): this.internal({field_name}: {field_name});",
                            name = self.name.as_ref(),
                            field_name = field.name.as_ref()
                        )?;
                    }
                } else {
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
                }
                writeln!(f)?;
                writeln!(f, "{indent}Map<String, dynamic> toJson() => {{")?;
                {
                    let indent = Indent(2);
                    for field in self.fields.iter() {
                        let to_json = ToJson {
                            name: field.name.as_ref(),
                            is_list: field.value_type.is_list,
                            scalar: &field.value_type.scalar,
                            assert_non_null: self.is_one_of || !field.value_type.is_required,
                        };
                        if to_json.assert_non_null {
                            writeln!(
                                f,
                                "{indent}if ({field_name} != null) '{serialized_field_name}': {to_json},",
                                field_name = field.name.as_ref(),
                                serialized_field_name = field.serialized_name,
                            )?;
                        } else {
                            writeln!(
                                f,
                                "{indent}'{serialized_field_name}': {to_json},",
                                serialized_field_name = field.serialized_name,
                            )?;
                        }
                    }
                }
                writeln!(f, "{indent}}};")?;
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
                                "{indent}{parent_name} to{parent_name}() => {parent_name}.internal({variant_name}: this);",
                                parent_name = parent.name.as_ref(),
                                variant_name = variant_name.as_ref(),
                            )?;
                        }
                    }
                }
            }
        }
        writeln!(f, "}}")
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
    assert_non_null: bool,
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
                if self.is_list {
                    write!(f, "{name}{non_null}.map((e) => e.toJson()).toList()")
                } else {
                    write!(f, "{name}{non_null}.toJson()")
                }
            }
        }
    }
}

impl fmt::Display for ObjectField {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let nullable = if self.value_type.is_required { "" } else { "?" };
        if self.value_type.is_list {
            write!(f, "List<{type}>{nullable} {name}", type = self.value_type.scalar.to_identifier(), name = self.name.as_ref())
        } else {
            write!(f, "{type}{nullable} {name}", type = self.value_type.scalar.to_identifier(), name = self.name.as_ref())
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::ast::{ScalarType, TypeReference};

    use super::*;

    #[test]
    fn empty_object() {
        let object = Object {
            name: Identifier::try_from("Test").unwrap(),
            description: Some(Comment::try_from("Test Object").unwrap()),
            fields: vec![],
            is_one_of: false,
            union_parents: vec![UnionParent::Union {
                parent: TypeReference {
                    name: Identifier::try_from("UnionParent").unwrap(),
                    path: "".into(),
                },
                variant_name: Identifier::try_from("test").unwrap(),
            }],
        };
        assert_eq!(
            object.to_string(),
            r"/// Test Object
class Test {
    Map<String, dynamic> toJson() => {};

    UnionParent toUnionParent() => UnionParent.internal(test: this);
}
"
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
            r"/// 주소 정보
class Address {
    final Country? country;
    /// **일반주소**
    final String addressLine1;
    /// **상세주소**
    final String addressLine2;
    /// **도시**
    final String? city;
    /// **주, 도, 시**
    final String? province;

    Address({
        this.country,
        required this.addressLine1,
        required this.addressLine2,
        this.city,
        this.province,
    });

    Map<String, dynamic> toJson() => {
        if (country != null) 'country': country!.toJson(),
        'addressLine1': addressLine1,
        'addressLine2': addressLine2,
        if (city != null) 'city': city!,
        if (province != null) 'province': province!,
    };
}
"
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
            r"/// **할부 개월 수 설정**
class MonthOption {
    /// **구매자가 선택할 수 없도록 고정된 할부 개월수**
    final int? fixedMonth;
    /// **구매자가 선택할 수 있는 할부 개월수 리스트**
    final List<int>? availableMonthList;

    MonthOption.internal({
        this.fixedMonth,
        this.availableMonthList,
    });
    MonthOption.fixedMonth(int fixedMonth): this.internal(fixedMonth: fixedMonth);
    MonthOption.availableMonthList(List<int> availableMonthList): this.internal(availableMonthList: availableMonthList);

    Map<String, dynamic> toJson() => {
        if (fixedMonth != null) 'fixedMonth': fixedMonth!,
        if (availableMonthList != null) 'availableMonthList': availableMonthList!,
    };
}
"
        );
    }
}
