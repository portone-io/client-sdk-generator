use std::fmt;

use super::{
    capitalize_first, Comment, CompositeType, Identifier, Indent, ScalarType, UnionParent,
};

#[derive(Debug, Clone)]
pub struct Object {
    pub name: Identifier,
    pub description: Option<Comment>,
    pub fields: Vec<ObjectField>,
    pub union_parents: Vec<UnionParent>,
    pub is_one_of: bool,
    pub skip_from_json: bool,
}

impl fmt::Display for Object {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for comment in self.description.iter().flat_map(Comment::lines) {
            writeln!(f, "/// {comment}")?;
        }
        if self.is_one_of {
            // Sealed class pattern
            writeln!(f, "sealed class {name} {{", name = self.name.as_ref())?;
            {
                let indent = Indent(1);
                writeln!(f, "{indent}Map<String, dynamic> toJson();")?;
                if !self.skip_from_json {
                    // fromJson static method for sealed class — dispatch by key
                    writeln!(
                        f,
                        "{indent}static {name} fromJson(Map<String, dynamic> json) {{",
                        name = self.name.as_ref()
                    )?;
                    {
                        let indent = Indent(2);
                        for field in self.fields.iter() {
                            let subclass_name = format!(
                                "{}{}",
                                self.name.as_ref(),
                                capitalize_first(field.name.as_ref())
                            );
                            writeln!(
                                f,
                                "{indent}if (json.containsKey('{serialized_name}')) return {subclass_name}.fromJson(json);",
                                serialized_name = field.serialized_name,
                            )?;
                        }
                        writeln!(
                            f,
                            "{indent}throw ArgumentError('Unknown {name} variant');",
                            name = self.name.as_ref()
                        )?;
                    }
                    let indent = Indent(1);
                    writeln!(f, "{indent}}}")?;
                }
            }
            writeln!(f, "}}")?;

            // Subclasses
            for field in self.fields.iter() {
                writeln!(f)?;
                let subclass_name = format!(
                    "{}{}",
                    self.name.as_ref(),
                    capitalize_first(field.name.as_ref())
                );
                for comment in field.description.iter().flat_map(Comment::lines) {
                    writeln!(f, "/// {comment}")?;
                }
                writeln!(
                    f,
                    "class {subclass_name} extends {name} {{",
                    name = self.name.as_ref()
                )?;
                {
                    let indent = Indent(1);
                    writeln!(f, "{indent}final {field};")?;
                    writeln!(
                        f,
                        "{indent}{subclass_name}(this.{field_name});",
                        field_name = field.name.as_ref()
                    )?;
                    if !self.skip_from_json {
                        // fromJson static method for subclass
                        let from_json = FromJson {
                            serialized_name: &field.serialized_name,
                            is_list: field.value_type.is_list,
                            scalar: &field.value_type.scalar,
                            is_required: field.value_type.is_required,
                        };
                        writeln!(
                            f,
                            "{indent}static {subclass_name} fromJson(Map<String, dynamic> json) =>"
                        )?;
                        {
                            let indent = Indent(3);
                            writeln!(f, "{indent}{subclass_name}({from_json});")?;
                        }
                    }
                    writeln!(f, "{indent}@override")?;
                    let to_json = ToJson {
                        name: field.name.as_ref(),
                        is_list: field.value_type.is_list,
                        scalar: &field.value_type.scalar,
                        assert_non_null: false,
                        null_aware_call: !field.value_type.is_required,
                    };
                    writeln!(
                        f,
                        "{indent}Map<String, dynamic> toJson() => {{'{serialized_name}': {to_json}}};",
                        serialized_name = field.serialized_name,
                    )?;
                }
                writeln!(f, "}}")?;
            }
            Ok(())
        } else {
            writeln!(f, "class {name} {{", name = self.name.as_ref())?;
            {
                let indent = Indent(1);
                if self.fields.is_empty() {
                    writeln!(f, "{indent}Map<String, dynamic> toJson() => {{}};")?;
                    if !self.skip_from_json {
                        writeln!(
                            f,
                            "{indent}static {name} fromJson(Map<String, dynamic> json) => {name}();",
                            name = self.name.as_ref()
                        )?;
                    }
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
                    if !self.skip_from_json {
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
}

#[derive(Debug, Clone)]
pub struct ObjectField {
    pub name: Identifier,
    pub serialized_name: String,
    pub value_type: CompositeType,
    pub description: Option<Comment>,
    pub import_alias: Option<String>,
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
                    // Int, Bool, String
                    let dart_type = scalar.to_identifier();
                    let nullable = if self.is_required { "" } else { "?" };
                    write!(f, "json['{key}'] as {dart_type}{nullable}")
                }
            }
        }
    }
}

impl fmt::Display for ObjectField {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let nullable = if self.value_type.is_required { "" } else { "?" };
        let type_prefix = match &self.import_alias {
            Some(alias) => format!("{alias}."),
            None => String::new(),
        };
        if self.value_type.is_list {
            write!(f, "List<{type_prefix}{type}>{nullable} {name}", type = self.value_type.scalar.to_identifier(), name = self.name.as_ref())
        } else {
            write!(f, "{type_prefix}{type}{nullable} {name}", type = self.value_type.scalar.to_identifier(), name = self.name.as_ref())
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
            skip_from_json: false,
        };
        assert_eq!(
            object.to_string(),
            r"/// Test Object
class Test {
    Map<String, dynamic> toJson() => {};
    static Test fromJson(Map<String, dynamic> json) => Test();

    UnionParent toUnionParent() => UnionParentTest(this);
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
                    import_alias: None,
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
                    import_alias: None,
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
                    import_alias: None,
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
                    import_alias: None,
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
                    import_alias: None,
                },
            ],
            is_one_of: false,
            union_parents: vec![],
            skip_from_json: false,
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

    static Address fromJson(Map<String, dynamic> json) => Address(
        country: json['country'] != null ? Country.fromJson(json['country']) : null,
        addressLine1: json['addressLine1'] as String,
        addressLine2: json['addressLine2'] as String,
        city: json['city'] as String?,
        province: json['province'] as String?,
    );
}
"
        );
    }

    #[test]
    fn one_of_object_with_nullable_type_reference() {
        let object = Object {
            name: Identifier::try_from("OfferPeriod").unwrap(),
            description: None,
            fields: vec![
                ObjectField {
                    name: Identifier::try_from("range").unwrap(),
                    serialized_name: "range".to_string(),
                    value_type: CompositeType {
                        scalar: ScalarType::TypeReference(TypeReference {
                            name: Identifier::try_from("OfferPeriodRange").unwrap(),
                            path: "".into(),
                        }),
                        is_list: false,
                        is_required: false,
                    },
                    description: None,
                    import_alias: Some("_offer_period_range".to_string()),
                },
                ObjectField {
                    name: Identifier::try_from("unit").unwrap(),
                    serialized_name: "unit".to_string(),
                    value_type: CompositeType {
                        scalar: ScalarType::TypeReference(TypeReference {
                            name: Identifier::try_from("OfferPeriodUnit").unwrap(),
                            path: "".into(),
                        }),
                        is_list: false,
                        is_required: true,
                    },
                    description: None,
                    import_alias: None,
                },
            ],
            is_one_of: true,
            union_parents: vec![],
            skip_from_json: true,
        };
        assert_eq!(
            object.to_string(),
            r"sealed class OfferPeriod {
    Map<String, dynamic> toJson();
}

class OfferPeriodRange extends OfferPeriod {
    final _offer_period_range.OfferPeriodRange? range;
    OfferPeriodRange(this.range);
    @override
    Map<String, dynamic> toJson() => {'range': range?.toJson()};
}

class OfferPeriodUnit extends OfferPeriod {
    final OfferPeriodUnit unit;
    OfferPeriodUnit(this.unit);
    @override
    Map<String, dynamic> toJson() => {'unit': unit.toJson()};
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
                    import_alias: None,
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
                    import_alias: None,
                },
            ],
            is_one_of: true,
            union_parents: vec![],
            skip_from_json: false,
        };
        assert_eq!(
            object.to_string(),
            r"/// **할부 개월 수 설정**
sealed class MonthOption {
    Map<String, dynamic> toJson();
    static MonthOption fromJson(Map<String, dynamic> json) {
        if (json.containsKey('fixedMonth')) return MonthOptionFixedMonth.fromJson(json);
        if (json.containsKey('availableMonthList')) return MonthOptionAvailableMonthList.fromJson(json);
        throw ArgumentError('Unknown MonthOption variant');
    }
}

/// **구매자가 선택할 수 없도록 고정된 할부 개월수**
class MonthOptionFixedMonth extends MonthOption {
    final int fixedMonth;
    MonthOptionFixedMonth(this.fixedMonth);
    static MonthOptionFixedMonth fromJson(Map<String, dynamic> json) =>
            MonthOptionFixedMonth(json['fixedMonth'] as int);
    @override
    Map<String, dynamic> toJson() => {'fixedMonth': fixedMonth};
}

/// **구매자가 선택할 수 있는 할부 개월수 리스트**
class MonthOptionAvailableMonthList extends MonthOption {
    final List<int> availableMonthList;
    MonthOptionAvailableMonthList(this.availableMonthList);
    static MonthOptionAvailableMonthList fromJson(Map<String, dynamic> json) =>
            MonthOptionAvailableMonthList((json['availableMonthList'] as List).cast<int>());
    @override
    Map<String, dynamic> toJson() => {'availableMonthList': availableMonthList};
}
"
        );
    }
}
