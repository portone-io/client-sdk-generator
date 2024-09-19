use indexmap::IndexMap;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Schema {
    /// PG사 식별자 목록
    pub pg_providers: IndexMap<String, PgProvider>,
    /// 리소스 목록
    pub resources: Resource,
    /// 메소드 목록
    pub methods: IndexMap<String, Method>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, JsonSchema)]
#[serde(rename_all = "camelCase", untagged)]
pub enum Resource {
    SubResources(IndexMap<String, Resource>),
    Parameter(Parameter),
}

#[derive(Serialize, Deserialize, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct PgProvider {
    /// PG사 설명
    pub description: String,
}

pub trait ParameterExt {
    fn name(&self) -> &str;
    fn description(&self) -> Option<&str>;
    fn r#type(&self) -> &ParameterType;
    fn optional(&self) -> bool;
    fn pg_specific(&self) -> Option<&IndexMap<String, ParameterType>>;
    fn deprecated(&self) -> bool;
}

impl ParameterExt for Parameter {
    fn name(&self) -> &str {
        match self {
            Parameter::Named(named) => &named.name,
            Parameter::Unnamed(_) => "",
        }
    }

    fn description(&self) -> Option<&str> {
        match self {
            Parameter::Named(named) => named.parameter.description.as_deref(),
            Parameter::Unnamed(parameter) => parameter.description.as_deref(),
        }
    }

    fn r#type(&self) -> &ParameterType {
        match self {
            Parameter::Named(named) => &named.parameter.r#type,
            Parameter::Unnamed(parameter) => &parameter.r#type,
        }
    }

    fn optional(&self) -> bool {
        match self {
            Parameter::Named(named) => named.parameter.optional,
            Parameter::Unnamed(parameter) => parameter.optional,
        }
    }

    fn pg_specific(&self) -> Option<&IndexMap<String, ParameterType>> {
        match self {
            Parameter::Named(named) => named.parameter.pg_specific.as_ref(),
            Parameter::Unnamed(parameter) => parameter.pg_specific.as_ref(),
        }
    }

    fn deprecated(&self) -> bool {
        match self {
            Parameter::Named(named) => named.parameter.deprecated,
            Parameter::Unnamed(parameter) => parameter.deprecated,
        }
    }
}

impl ParameterExt for NamedParameter {
    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> Option<&str> {
        self.parameter.description.as_deref()
    }

    fn r#type(&self) -> &ParameterType {
        &self.parameter.r#type
    }

    fn optional(&self) -> bool {
        self.parameter.optional
    }

    fn pg_specific(&self) -> Option<&IndexMap<String, ParameterType>> {
        self.parameter.pg_specific.as_ref()
    }

    fn deprecated(&self) -> bool {
        self.parameter.deprecated
    }
}

impl ParameterExt for UnnamedParameter {
    fn name(&self) -> &str {
        ""
    }

    fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    fn r#type(&self) -> &ParameterType {
        &self.r#type
    }

    fn optional(&self) -> bool {
        self.optional
    }

    fn pg_specific(&self) -> Option<&IndexMap<String, ParameterType>> {
        self.pg_specific.as_ref()
    }

    fn deprecated(&self) -> bool {
        self.deprecated
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, JsonSchema)]
#[serde(rename_all = "camelCase", untagged)]
pub enum Parameter {
    Named(NamedParameter),
    Unnamed(UnnamedParameter),
}

#[derive(Serialize, Deserialize, Debug, Default, PartialEq, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct NamedParameter {
    /// 파라미터 이름
    name: String,
    #[serde(flatten)]
    pub parameter: UnnamedParameter,
}

impl NamedParameter {
    pub fn new(name: String, parameter: UnnamedParameter) -> Self {
        Self { name, parameter }
    }
}

#[derive(Serialize, Deserialize, Debug, Default, PartialEq, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct UnnamedParameter {
    /// 파라미터 설명
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    #[serde(flatten)]
    r#type: ParameterType,
    /// Optional 여부
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    optional: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pg_specific: Option<IndexMap<String, ParameterType>>,
    /// Deprecated 여부
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    deprecated: bool,
}

impl UnnamedParameter {
    pub fn new(
        description: Option<String>,
        r#type: ParameterType,
        optional: bool,
        pg_specific: Option<IndexMap<String, ParameterType>>,
        deprecated: bool,
    ) -> Self {
        Self {
            description,
            r#type,
            optional,
            pg_specific,
            deprecated,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Default, PartialEq, Clone, JsonSchema)]
#[serde(tag = "type", rename_all = "camelCase")]
/// 파라미터 타입
pub enum ParameterType {
    #[schemars(title = "string")]
    String,
    #[schemars(title = "stringLiteral")]
    StringLiteral {
        /// StringLiteral의 값
        value: String,
    },
    #[schemars(title = "integer")]
    Integer,
    #[schemars(title = "boolean")]
    Boolean,
    #[schemars(title = "array")]
    Array {
        /// Array의 item 타입
        items: Box<Parameter>,
    },
    #[schemars(title = "object")]
    Object {
        /// Object의 프로퍼티 목록
        properties: IndexMap<String, Parameter>,
    },
    #[schemars(title = "emptyObject")]
    EmptyObject,
    #[schemars(title = "enum")]
    Enum {
        /// Enum의 variant 목록
        variants: IndexMap<String, EnumVariant>,
        #[serde(skip_serializing_if = "Option::is_none")]
        value_prefix: Option<String>,
    },
    #[schemars(title = "oneOf")]
    OneOf {
        /// OneOf의 타입 목록
        properties: IndexMap<String, Parameter>,
    },
    #[schemars(title = "union")]
    Union {
        /// Union의 타입 목록
        types: Vec<Parameter>,
    },
    #[schemars(title = "intersection")]
    Intersection {
        /// Intersection의 타입 목록
        types: Vec<Parameter>,
    },
    #[schemars(title = "discriminatedUnion")]
    DiscriminatedUnion {
        /// DiscriminatedUnion의 타입 목록
        types: IndexMap<String, Parameter>,
        /// Discriminator 프로퍼티 이름
        discriminator: String,
    },
    #[schemars(title = "resourceRef")]
    ResourceRef(ResourceRef),
    #[schemars(title = "error")]
    #[serde(rename_all = "camelCase")]
    Error {
        #[serde(skip_serializing_if = "Option::is_none")]
        transaction_type: Option<String>,
        /// Error의 프로퍼티 목록
        properties: IndexMap<String, Parameter>,
    },
    #[schemars(title = "unknown")]
    #[default]
    Unknown,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ResourceRef {
    #[serde(rename = "$ref")]
    resource_ref: String,
}

impl ResourceRef {
    pub fn new(resource_ref: &str) -> Self {
        Self {
            resource_ref: resource_ref.to_string(),
        }
    }

    pub fn resource_ref(&self) -> &str {
        self.resource_ref.trim_start_matches("#/resources/")
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct EnumVariant {
    /// Enum variant 설명
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Method {
    /// 메소드 설명
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// 메소드 입력 파라미터
    pub input: Parameter,
    /// 메소드 콜백 목록
    pub callbacks: Option<IndexMap<String, Callback>>,
    /// 메소드 출력 파라미터
    pub output: Option<Parameter>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Callback {
    /// 콜백 설명
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// 콜백 입력 파라미터
    pub input: IndexMap<String, Parameter>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_schema_serialization_to_yaml() {
        let mut pg_providers: IndexMap<String, PgProvider> = IndexMap::new();
        pg_providers.insert(
            "paypal".to_string(),
            PgProvider {
                description: "PayPal payment provider".to_string(),
            },
        );
        pg_providers.insert(
            "stripe".to_string(),
            PgProvider {
                description: "Stripe payment provider".to_string(),
            },
        );
        pg_providers.insert(
            "tosspayments".to_string(),
            PgProvider {
                description: "Toss Payments provider".to_string(),
            },
        );
        pg_providers.insert(
            "danal".to_string(),
            PgProvider {
                description: "Danal payment provider".to_string(),
            },
        );
        pg_providers.insert(
            "kcp".to_string(),
            PgProvider {
                description: "KCP payment provider".to_string(),
            },
        );

        let mut parameters: IndexMap<String, Parameter> = IndexMap::new();
        parameters.insert(
            "Person".to_string(),
            Parameter::Unnamed(UnnamedParameter {
                description: Some("A person object".to_string()),
                r#type: ParameterType::Object {
                    properties: {
                        let mut properties = IndexMap::new();
                        properties.insert(
                            "name".to_string(),
                            Parameter::Unnamed(UnnamedParameter {
                                description: Some("The person's name".to_string()),
                                r#type: ParameterType::String,
                                optional: false,
                                pg_specific: None,
                                deprecated: false,
                            }),
                        );
                        properties.insert(
                            "age".to_string(),
                            Parameter::Unnamed(UnnamedParameter {
                                description: Some("The person's age".to_string()),
                                r#type: ParameterType::Integer,
                                optional: true,
                                pg_specific: None,
                                deprecated: false,
                            }),
                        );
                        properties
                    },
                },
                optional: false,
                pg_specific: None,
                deprecated: false,
            }),
        );
        parameters.insert(
            "Colors".to_string(),
            Parameter::Unnamed(UnnamedParameter {
                description: Some("An enum of colors".to_string()),
                r#type: ParameterType::Enum {
                    variants: {
                        let mut variants = IndexMap::new();
                        variants.insert(
                            "Red".to_string(),
                            EnumVariant {
                                description: Some("Red color".to_string()),
                            },
                        );
                        variants.insert(
                            "Green".to_string(),
                            EnumVariant {
                                description: Some("Green color".to_string()),
                            },
                        );
                        variants.insert(
                            "Blue".to_string(),
                            EnumVariant {
                                description: Some("Blue color".to_string()),
                            },
                        );
                        variants
                    },
                    value_prefix: None,
                },
                optional: false,
                pg_specific: None,
                deprecated: false,
            }),
        );
        parameters.insert(
            "Numbers".to_string(),
            Parameter::Unnamed(UnnamedParameter {
                description: Some("An array of numbers".to_string()),
                r#type: ParameterType::Array {
                    items: Box::new(Parameter::Unnamed(UnnamedParameter {
                        r#type: ParameterType::Integer,
                        ..Default::default()
                    })),
                },
                optional: false,
                pg_specific: None,
                deprecated: false,
            }),
        );
        parameters.insert(
            "IsHuman".to_string(),
            Parameter::Unnamed(UnnamedParameter {
                description: Some("Boolean to indicate if human".to_string()),
                r#type: ParameterType::Boolean,
                optional: false,
                pg_specific: None,
                deprecated: false,
            }),
        );
        parameters.insert(
            "Reference".to_string(),
            Parameter::Unnamed(UnnamedParameter {
                description: Some("Reference to another parameter".to_string()),
                r#type: ParameterType::ResourceRef(ResourceRef {
                    resource_ref: "#/resources/Person".to_string(),
                }),
                optional: false,
                pg_specific: None,
                deprecated: false,
            }),
        );

        let schema = Schema {
            pg_providers,
            resources: Resource::SubResources(
                parameters
                    .into_iter()
                    .map(|(k, v)| (k, Resource::Parameter(v)))
                    .collect(),
            ),
            methods: IndexMap::new(),
        };

        // Serialize the schema to YAML
        let serialized = serde_yaml::to_string(&schema).unwrap();

        // Expected YAML
        let expected_yaml = r#"
pgProviders:
  paypal:
    description: PayPal payment provider
  stripe:
    description: Stripe payment provider
  tosspayments:
    description: Toss Payments provider
  danal:
    description: Danal payment provider
  kcp:
    description: KCP payment provider
resources:
  Person:
    description: A person object
    type: object
    properties:
      name:
        description: The person's name
        type: string
      age:
        description: The person's age
        type: integer
        optional: true
  Colors:
    description: An enum of colors
    type: enum
    variants:
      Red:
        description: Red color
      Green:
        description: Green color
      Blue:
        description: Blue color
  Numbers:
    description: An array of numbers
    type: array
    items:
      type: integer
  IsHuman:
    description: Boolean to indicate if human
    type: boolean
  Reference:
    description: Reference to another parameter
    type: resourceRef
    $ref: '#/resources/Person'
methods: {}
"#;

        // Compare the serialized schema with the expected YAML
        assert_eq!(serialized.trim(), expected_yaml.trim());

        // Deserialize the YAML back to schema
        let deserialized: Schema = serde_yaml::from_str(&serialized).unwrap();

        // Compare the deserialized schema with the original schema
        assert_eq!(deserialized, schema);
    }
}
