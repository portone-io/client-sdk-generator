use std::fmt;

use super::{Comment, Identifier, Indent, UnionParent};

#[derive(Debug, Clone)]
pub struct Object {
    pub name: Identifier,
    pub description: Option<Comment>,
    pub fields: Vec<ObjectField>,
    pub union_parents: Vec<UnionParent>,
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
                writeln!(f, "{indent}Map<String, dynamic> _toJson() => {{}};")?;
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
                writeln!(f, "{indent}Map<String, dynamic> _toJson() => {{")?;
                {
                    let indent = Indent(2);
                    for field in self.fields.iter() {
                        if field.value_type.is_required {
                            writeln!(
                                f,
                                "{indent}'{field_name}': {field_name}._toJson(),",
                                field_name = field.name.as_ref()
                            )?;
                        } else {
                            writeln!(
                                f,
                                "{indent}if ({field_name} != null) '{field_name}': {field_name}._toJson(),",
                                field_name = field.name.as_ref()
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
                        UnionParent::Union { parent_name, variant_name } => {
                            writeln!(
                                f,
                                "{indent}{parent_name} to{parent_name}() => {parent_name}._internal({variant_name}: this);",
                                parent_name = parent_name.as_ref(),
                                variant_name = variant_name.as_ref(),
                            )?;
                        }
                        UnionParent::DiscriminatedUnion { parent_name, variant_name, discriminator_value } => {
                            writeln!(
                                f,
                                "{indent}{parent_name} to{parent_name}() => {parent_name}._internal('{discriminator_value}', {variant_name}: this);",
                                parent_name = parent_name.as_ref(),
                                variant_name = variant_name.as_ref(),
                                discriminator_value = discriminator_value,
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
    pub value_type: ValueType,
    pub description: Option<Comment>,
}

impl fmt::Display for ObjectField {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let nullable = if self.value_type.is_required { "" } else { "?" };
        if self.value_type.is_list {
            write!(f, "List<{type}{nullable}> {name}", type = self.value_type.name.as_ref(), name = self.name.as_ref())
        } else {
            write!(f, "{type}{nullable} {name}", type = self.value_type.name.as_ref(), name = self.name.as_ref())
        }
    }
}

#[derive(Debug, Clone)]
pub struct ValueType {
    pub name: Identifier,
    pub is_list: bool,
    pub is_required: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_object() {
        let object = Object {
            name: Identifier::try_from("Test").unwrap(),
            description: Some(Comment("Test Object".into())),
            fields: vec![],
            union_parents: vec![UnionParent::Union { parent_name: Identifier::try_from("UnionParent").unwrap(), variant_name: Identifier::try_from("test").unwrap() }],
        };
        assert_eq!(
            object.to_string(),
            r"/// Test Object
class Test {
    Map<String, dynamic> _toJson() => {};

    UnionParent toUnionParent() => UnionParent._internal(test: this);
}
"
        );
    }

    #[test]
    fn object_with_fields() {
        let object = Object {
            name: Identifier::try_from("Address").unwrap(),
            description: Some(Comment("주소 정보".into())),
            fields: vec![
                ObjectField {
                    name: Identifier::try_from("country").unwrap(),
                    value_type: ValueType {
                        name: Identifier::try_from("Country").unwrap(),
                        is_list: false,
                        is_required: false,
                    },
                    description: None,
                },
                ObjectField {
                    name: Identifier::try_from("addressLine1").unwrap(),
                    value_type: ValueType {
                        name: Identifier::try_from("String").unwrap(),
                        is_list: false,
                        is_required: true,
                    },
                    description: Some(Comment("**일반주소**".into())),
                },
                ObjectField {
                    name: Identifier::try_from("addressLine2").unwrap(),
                    value_type: ValueType {
                        name: Identifier::try_from("String").unwrap(),
                        is_list: false,
                        is_required: true,
                    },
                    description: Some(Comment("**상세주소**".into())),
                },
                ObjectField {
                    name: Identifier::try_from("city").unwrap(),
                    value_type: ValueType {
                        name: Identifier::try_from("String").unwrap(),
                        is_list: false,
                        is_required: false,
                    },
                    description: Some(Comment("**도시**".into())),
                },
                ObjectField {
                    name: Identifier::try_from("province").unwrap(),
                    value_type: ValueType {
                        name: Identifier::try_from("String").unwrap(),
                        is_list: false,
                        is_required: false,
                    },
                    description: Some(Comment("**주, 도, 시**".into())),
                },
            ],
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

    Map<String, dynamic> _toJson() => {
        if (country != null) 'country': country._toJson(),
        'addressLine1': addressLine1._toJson(),
        'addressLine2': addressLine2._toJson(),
        if (city != null) 'city': city._toJson(),
        if (province != null) 'province': province._toJson(),
    };
}
"
        );
    }
}
