use indexmap::IndexMap;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

better_scoped_tls::scoped_tls!(pub static RESOURCE_INDEX: IndexMap<String, Parameter>);

#[derive(Serialize, Deserialize, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Schema {
    /// 플래그 목록
    pub flags: IndexMap<String, Flag>,
    /// 리소스 목록
    pub resources: Resource,
    /// 메소드 목록
    pub methods: IndexMap<String, Method>,
}

impl Schema {
    pub fn build_resource_index(&self) -> IndexMap<String, Parameter> {
        let mut index = IndexMap::new();
        Schema::collect_resources("", &self.resources, &mut index);
        index
    }

    fn collect_resources(path: &str, resource: &Resource, index: &mut IndexMap<String, Parameter>) {
        match resource {
            Resource::SubResources(sub_resources) => {
                for (name, sub_resource) in sub_resources {
                    let new_path = if path.is_empty() {
                        name.clone()
                    } else {
                        format!("{path}/{name}")
                    };
                    Schema::collect_resources(&new_path, sub_resource, index);
                }
            }
            Resource::Parameter(parameter) => {
                index.insert(path.to_string(), parameter.clone());
            }
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, JsonSchema)]
#[serde(rename_all = "camelCase", untagged)]
pub enum Resource {
    SubResources(IndexMap<String, Resource>),
    Parameter(Parameter),
}

#[derive(Serialize, Deserialize, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Flag {
    /// 플래그 설명
    pub description: String,
}

pub trait ParameterExt {
    fn name(&self) -> Option<&str>;
    fn description(&self) -> Option<String>;
    fn r#type(&self) -> &ParameterType;
    fn optional(&self) -> bool;
    fn flag_options(&self) -> Option<&IndexMap<String, FlagOption>>;
    fn deprecated(&self) -> bool;
}

impl ParameterExt for Parameter {
    fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    fn description(&self) -> Option<String> {
        match (self.description.as_deref(), &self.r#type) {
            (None, ParameterType::ResourceRef(resource_ref)) => RESOURCE_INDEX.with(|index| {
                index
                    .get(resource_ref.resource_ref())
                    .and_then(|parameter| parameter.description().map(|s| s.to_string()))
            }),
            (
                None,
                ParameterType::Array {
                    items,
                    hide_if_empty: _,
                },
            ) => items.description(),
            (description, _) => description.map(|s| s.to_string()),
        }
    }

    fn r#type(&self) -> &ParameterType {
        &self.r#type
    }

    fn optional(&self) -> bool {
        self.optional
    }

    fn flag_options(&self) -> Option<&IndexMap<String, FlagOption>> {
        self.flag_options.as_ref()
    }

    fn deprecated(&self) -> bool {
        match (self.deprecated, &self.r#type) {
            (true, _) => true,
            (_, ParameterType::ResourceRef(resource_ref)) => RESOURCE_INDEX.with(|index| {
                index
                    .get(resource_ref.resource_ref())
                    .map(|parameter| parameter.deprecated())
                    .unwrap_or(false)
            }),
            (
                _,
                ParameterType::Array {
                    items,
                    hide_if_empty: _,
                },
            ) => items.deprecated(),
            _ => false,
        }
    }
}

#[derive(Default, Serialize, Deserialize, Debug, PartialEq, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Parameter {
    /// 파라미터 이름
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// 파라미터 설명
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(flatten)]
    pub r#type: ParameterType,
    /// Optional 여부
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub optional: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flag_options: Option<IndexMap<String, FlagOption>>,
    /// Deprecated 여부
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub deprecated: bool,
}

impl Parameter {
    pub fn new(
        name: Option<String>,
        description: Option<String>,
        r#type: ParameterType,
        optional: bool,
        flag_options: Option<IndexMap<String, FlagOption>>,
        deprecated: bool,
    ) -> Self {
        Self {
            name,
            description,
            r#type,
            optional,
            flag_options,
            deprecated,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct FlagOption {
    /// Visible 여부
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    visible: bool,
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
        /// Array가 비어있을 때 숨기기 여부
        #[serde(default, skip_serializing_if = "std::ops::Not::not")]
        hide_if_empty: bool,
    },
    #[schemars(title = "object")]
    #[serde(rename_all = "camelCase")]
    Object {
        /// Object의 프로퍼티 목록
        properties: IndexMap<String, Parameter>,
        /// Object가 비어있을 때 숨기기 여부
        #[serde(default, skip_serializing_if = "std::ops::Not::not")]
        hide_if_empty: bool,
    },
    #[schemars(title = "emptyObject")]
    EmptyObject,
    #[schemars(title = "enum")]
    #[serde(rename_all = "camelCase")]
    Enum {
        /// Enum의 variant 목록
        variants: IndexMap<String, EnumVariant>,
        #[serde(skip_serializing_if = "Option::is_none")]
        value_prefix: Option<String>,
    },
    #[schemars(title = "oneOf")]
    #[serde(rename_all = "camelCase")]
    OneOf {
        /// OneOf의 타입 목록
        properties: IndexMap<String, Parameter>,
        /// OneOf가 비어있을 때 숨기기 여부
        #[serde(default, skip_serializing_if = "std::ops::Not::not")]
        hide_if_empty: bool,
    },
    #[schemars(title = "union")]
    #[serde(rename_all = "camelCase")]
    Union {
        /// Union의 타입 목록
        types: Vec<Parameter>,
        /// Union이 비어있을 때 숨기기 여부
        #[serde(default, skip_serializing_if = "std::ops::Not::not")]
        hide_if_empty: bool,
    },
    #[schemars(title = "intersection")]
    #[serde(rename_all = "camelCase")]
    Intersection {
        /// Intersection의 타입 목록
        types: Vec<Parameter>,
        /// Intersection이 비어있을 때 숨기기 여부
        #[serde(default, skip_serializing_if = "std::ops::Not::not")]
        hide_if_empty: bool,
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
    #[schemars(title = "json")]
    #[default]
    Json,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alias: Option<String>,
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
        let mut flags: IndexMap<String, Flag> = IndexMap::new();
        flags.insert(
            "paypal".to_string(),
            Flag {
                description: "PayPal payment provider".to_string(),
            },
        );
        flags.insert(
            "stripe".to_string(),
            Flag {
                description: "Stripe payment provider".to_string(),
            },
        );
        flags.insert(
            "tosspayments".to_string(),
            Flag {
                description: "Toss Payments provider".to_string(),
            },
        );
        flags.insert(
            "danal".to_string(),
            Flag {
                description: "Danal payment provider".to_string(),
            },
        );
        flags.insert(
            "kcp".to_string(),
            Flag {
                description: "KCP payment provider".to_string(),
            },
        );

        let mut parameters: IndexMap<String, Parameter> = IndexMap::new();
        parameters.insert(
            "Person".to_string(),
            Parameter {
                name: None,
                description: Some("A person object".to_string()),
                r#type: ParameterType::Object {
                    hide_if_empty: false,
                    properties: {
                        let mut properties = IndexMap::new();
                        properties.insert(
                            "name".to_string(),
                            Parameter {
                                name: None,
                                description: Some("The person's name".to_string()),
                                r#type: ParameterType::String,
                                optional: false,
                                flag_options: None,
                                deprecated: false,
                            },
                        );
                        properties.insert(
                            "age".to_string(),
                            Parameter {
                                name: None,
                                description: Some("The person's age".to_string()),
                                r#type: ParameterType::Integer,
                                optional: true,
                                flag_options: None,
                                deprecated: false,
                            },
                        );
                        properties
                    },
                },
                optional: false,
                flag_options: None,
                deprecated: false,
            },
        );
        parameters.insert(
            "Colors".to_string(),
            Parameter {
                name: None,
                description: Some("An enum of colors".to_string()),
                r#type: ParameterType::Enum {
                    variants: {
                        let mut variants = IndexMap::new();
                        variants.insert(
                            "Red".to_string(),
                            EnumVariant {
                                description: Some("Red color".to_string()),
                                alias: None,
                            },
                        );
                        variants.insert(
                            "Green".to_string(),
                            EnumVariant {
                                description: Some("Green color".to_string()),
                                alias: None,
                            },
                        );
                        variants.insert(
                            "Blue".to_string(),
                            EnumVariant {
                                description: Some("Blue color".to_string()),
                                alias: Some("Aqua".to_string()),
                            },
                        );
                        variants
                    },
                    value_prefix: None,
                },
                optional: false,
                flag_options: None,
                deprecated: false,
            },
        );
        parameters.insert(
            "Numbers".to_string(),
            Parameter {
                name: None,
                description: Some("An array of numbers".to_string()),
                r#type: ParameterType::Array {
                    items: Box::new(Parameter {
                        name: None,
                        description: None,
                        r#type: ParameterType::Integer,
                        optional: false,
                        flag_options: None,
                        deprecated: false,
                    }),
                    hide_if_empty: false,
                },
                optional: false,
                flag_options: None,
                deprecated: false,
            },
        );
        parameters.insert(
            "IsHuman".to_string(),
            Parameter {
                name: None,
                description: Some("Boolean to indicate if human".to_string()),
                r#type: ParameterType::Boolean,
                optional: false,
                flag_options: None,
                deprecated: false,
            },
        );
        parameters.insert(
            "Reference".to_string(),
            Parameter {
                name: None,
                description: Some("Reference to another parameter".to_string()),
                r#type: ParameterType::ResourceRef(ResourceRef {
                    resource_ref: "#/resources/Person".to_string(),
                }),
                optional: false,
                flag_options: None,
                deprecated: false,
            },
        );

        let schema = Schema {
            flags,
            resources: Resource::SubResources(
                parameters
                    .into_iter()
                    .map(|(k, v)| (k, Resource::Parameter(v)))
                    .collect(),
            ),
            methods: IndexMap::new(),
        };

        // Serialize the schema to YAML
        let serialized = serde_yaml_ng::to_string(&schema).unwrap();

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
        alias: Aqua
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
        let deserialized: Schema = serde_yaml_ng::from_str(&serialized).unwrap();

        // Compare the deserialized schema with the original schema
        assert_eq!(deserialized, schema);
    }
}
