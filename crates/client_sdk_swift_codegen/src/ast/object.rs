use std::fmt;

use super::{Comment, CompositeType, Identifier, Indent, UnionParent};

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
            for line in desc.lines() {
                writeln!(f, "/// {line}")?;
            }
        }

        if self.fields.is_empty() && !self.is_one_of {
            // Empty object case
            writeln!(
                f,
                "public struct {name}: Codable {{",
                name = self.name.as_ref()
            )?;
            {
                let indent = Indent(1);
                writeln!(f, "{indent}public init() {{}}")?;
            }
            writeln!(f, "}}")
        } else if self.is_one_of {
            // OneOf (enum with associated values) case
            writeln!(
                f,
                "public enum {name}: Codable {{",
                name = self.name.as_ref()
            )?;
            {
                let indent = Indent(1);
                for field in self.fields.iter() {
                    if let Some(ref desc) = field.description {
                        for line in desc.lines() {
                            writeln!(f, "{indent}/// {line}")?;
                        }
                    }
                    let field_type = if field.value_type.is_list {
                        format!("[{}]", field.value_type.scalar.to_swift_type())
                    } else {
                        field.value_type.scalar.to_swift_type().to_string()
                    };
                    writeln!(
                        f,
                        "{indent}case {field_name}({field_type})",
                        field_name = field.name.as_ref()
                    )?;
                }
                writeln!(f)?;

                // CodingKeys
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

                // Custom Decodable init
                writeln!(f, "{indent}public init(from decoder: Decoder) throws {{")?;
                {
                    let indent = Indent(2);
                    writeln!(
                        f,
                        "{indent}let container = try decoder.container(keyedBy: CodingKeys.self)"
                    )?;
                    for (i, field) in self.fields.iter().enumerate() {
                        let field_type = if field.value_type.is_list {
                            format!("[{}]", field.value_type.scalar.to_swift_type())
                        } else {
                            field.value_type.scalar.to_swift_type().to_string()
                        };
                        if i == 0 {
                            writeln!(
                                f,
                                "{indent}if let value = try container.decodeIfPresent({field_type}.self, forKey: .{field_name}) {{",
                                field_name = field.name.as_ref()
                            )?;
                        } else {
                            writeln!(
                                f,
                                "{indent}}} else if let value = try container.decodeIfPresent({field_type}.self, forKey: .{field_name}) {{",
                                field_name = field.name.as_ref()
                            )?;
                        }
                        {
                            let indent = Indent(3);
                            writeln!(
                                f,
                                "{indent}self = .{field_name}(value)",
                                field_name = field.name.as_ref()
                            )?;
                        }
                    }
                    writeln!(f, "{indent}}} else {{")?;
                    {
                        let indent = Indent(3);
                        writeln!(
                            f,
                            "{indent}throw DecodingError.dataCorrupted(DecodingError.Context(codingPath: decoder.codingPath, debugDescription: \"No valid case found\"))"
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
                    writeln!(
                        f,
                        "{indent}var container = encoder.container(keyedBy: CodingKeys.self)"
                    )?;
                    writeln!(f, "{indent}switch self {{")?;
                    for field in self.fields.iter() {
                        let indent = Indent(2);
                        writeln!(
                            f,
                            "{indent}case .{field_name}(let value):",
                            field_name = field.name.as_ref()
                        )?;
                        {
                            let indent = Indent(3);
                            writeln!(
                                f,
                                "{indent}try container.encode(value, forKey: .{field_name})",
                                field_name = field.name.as_ref()
                            )?;
                        }
                    }
                    writeln!(f, "{indent}}}")?;
                }
                writeln!(f, "{indent}}}")?;
            }
            writeln!(f, "}}")
        } else {
            // Regular struct case
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
}

#[derive(Debug, Clone)]
pub struct ObjectField {
    pub name: Identifier,
    pub serialized_name: String,
    pub value_type: CompositeType,
    pub description: Option<Comment>,
}

impl fmt::Display for ObjectField {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let nullable = if self.value_type.is_required { "" } else { "?" };
        let field_type = if self.value_type.is_list {
            format!("[{}]", self.value_type.scalar.to_swift_type())
        } else {
            self.value_type.scalar.to_swift_type().to_string()
        };

        write!(
            f,
            "public let {name}: {field_type}{nullable}",
            name = self.name.as_ref()
        )
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
            r#"public struct IssueBillingKeyRequestUnionPaypal: Codable {
    public init() {}
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
            r#"/// 주소 정보
public struct Address: Codable {
    public let country: Country?
    /// **일반주소**
    public let addressLine1: String
    /// **상세주소**
    public let addressLine2: String
    /// **도시**
    public let city: String?
    /// **주, 도, 시**
    public let province: String?

    public init(country: Country? = nil, addressLine1: String, addressLine2: String, city: String? = nil, province: String? = nil) {
        self.country = country
        self.addressLine1 = addressLine1
        self.addressLine2 = addressLine2
        self.city = city
        self.province = province
    }
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
            r#"/// **할부 개월 수 설정**
public enum MonthOption: Codable {
    /// **구매자가 선택할 수 없도록 고정된 할부 개월수**
    case fixedMonth(Int)
    /// **구매자가 선택할 수 있는 할부 개월수 리스트**
    case availableMonthList([Int])

    private enum CodingKeys: String, CodingKey {
        case fixedMonth
        case availableMonthList
    }

    public init(from decoder: Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)
        if let value = try container.decodeIfPresent(Int.self, forKey: .fixedMonth) {
            self = .fixedMonth(value)
        } else if let value = try container.decodeIfPresent([Int].self, forKey: .availableMonthList) {
            self = .availableMonthList(value)
        } else {
            throw DecodingError.dataCorrupted(DecodingError.Context(codingPath: decoder.codingPath, debugDescription: "No valid case found"))
        }
    }

    public func encode(to encoder: Encoder) throws {
        var container = encoder.container(keyedBy: CodingKeys.self)
        switch self {
        case .fixedMonth(let value):
            try container.encode(value, forKey: .fixedMonth)
        case .availableMonthList(let value):
            try container.encode(value, forKey: .availableMonthList)
        }
    }
}
"#
        );
    }

    #[test]
    fn object_with_json_field() {
        let object = Object {
            name: Identifier::try_from("CustomData").unwrap(),
            description: Some(Comment::try_from("커스텀 데이터").unwrap()),
            fields: vec![
                ObjectField {
                    name: Identifier::try_from("id").unwrap(),
                    serialized_name: "id".to_string(),
                    value_type: CompositeType {
                        scalar: ScalarType::String,
                        is_list: false,
                        is_required: true,
                    },
                    description: None,
                },
                ObjectField {
                    name: Identifier::try_from("metadata").unwrap(),
                    serialized_name: "metadata".to_string(),
                    value_type: CompositeType {
                        scalar: ScalarType::Json,
                        is_list: false,
                        is_required: true,
                    },
                    description: Some(Comment::try_from("**추가 메타데이터**").unwrap()),
                },
                ObjectField {
                    name: Identifier::try_from("tags").unwrap(),
                    serialized_name: "tags".to_string(),
                    value_type: CompositeType {
                        scalar: ScalarType::String,
                        is_list: true,
                        is_required: false,
                    },
                    description: None,
                },
            ],
            is_one_of: false,
            union_parents: vec![],
        };
        assert_eq!(
            object.to_string(),
            r#"/// 커스텀 데이터
public struct CustomData: Codable {
    public let id: String
    /// **추가 메타데이터**
    public let metadata: [String: Any]
    public let tags: [String]?

    public init(id: String, metadata: [String: Any], tags: [String]? = nil) {
        self.id = id
        self.metadata = metadata
        self.tags = tags
    }
}
"#
        );
    }
}
