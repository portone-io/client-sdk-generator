use std::fmt::Write;
use std::path::PathBuf;

use biome_js_syntax::{AnyJsDeclaration, JsVariableDeclaration};
use browser_sdk_schema::{self as schema, ParameterExt, RESOURCE_INDEX};
use browser_sdk_ts_codegen_macros::ts_parse;
use convert_case::Casing;
use indexmap::{IndexMap, IndexSet};

use crate::{import::resource_ref_to_path, ImportEntry};

use super::comment::JsDocExt;

pub(crate) fn generate_parameter(
    parameter: &schema::Parameter,
    decls: &mut Vec<AnyJsDeclaration>,
    imports: &mut IndexSet<ImportEntry>,
    parent_name: &str,
    current_module_path: &PathBuf,
    resource_base_path: &PathBuf,
) -> String {
    if parameter.name().is_some() {
        generate_named_parameter(
            parameter,
            decls,
            imports,
            parent_name,
            current_module_path,
            resource_base_path,
        )
    } else {
        generate_unnamed_parameter(
            parameter,
            decls,
            imports,
            parent_name,
            current_module_path,
            resource_base_path,
        )
    }
}

pub(crate) fn generate_named_parameter(
    parameter: &schema::Parameter,
    decls: &mut Vec<AnyJsDeclaration>,
    imports: &mut IndexSet<ImportEntry>,
    _parent_name: &str,
    current_module_path: &PathBuf,
    resource_base_path: &PathBuf,
) -> String {
    let type_name = parameter.name().unwrap_or_default();
    let type_def = generate_unnamed_parameter(
        parameter,
        decls,
        imports,
        type_name,
        current_module_path,
        resource_base_path,
    );
    let description = parameter.description().to_jsdoc(parameter.deprecated());

    match parameter.r#type() {
        schema::ParameterType::Error { .. } => (),
        _ => {
            let type_alias =
                ts_parse!("{description}type {type_name} = {type_def};" as TsTypeAliasDeclaration);

            decls.push(type_alias.into());
        }
    }

    type_name.to_string()
}

pub(crate) fn generate_unnamed_parameter(
    parameter: &schema::Parameter,
    decls: &mut Vec<AnyJsDeclaration>,
    imports: &mut IndexSet<ImportEntry>,
    parent_name: &str,
    current_module_path: &PathBuf,
    resource_base_path: &PathBuf,
) -> String {
    if let Some(const_decl) = generate_const_enum_declaration(
        parent_name,
        parameter.description().as_deref(),
        parameter.r#type(),
    ) {
        decls.push(const_decl.into());
    }
    let mut type_def = generate_parameter_type(
        parameter.r#type(),
        decls,
        imports,
        parent_name,
        current_module_path,
        resource_base_path,
    );
    if parameter.optional() {
        type_def.push_str(" | undefined");
    }
    type_def
}

fn generate_parameter_type(
    parameter_type: &schema::ParameterType,
    decls: &mut Vec<AnyJsDeclaration>,
    imports: &mut IndexSet<ImportEntry>,
    parent_name: &str,
    current_module_path: &PathBuf, // 현재 모듈의 경로
    resource_base_path: &PathBuf,  // 리소스의 기본 경로 (예: 프로젝트 루트)
) -> String {
    match parameter_type {
        schema::ParameterType::String => String::from("string"),
        schema::ParameterType::StringLiteral { value } => format!("'{}'", value),
        schema::ParameterType::Integer => String::from("number"),
        schema::ParameterType::Boolean => String::from("boolean"),
        schema::ParameterType::Array { items } => {
            let item_type = generate_parameter(
                items,
                decls,
                imports,
                &format!("{}Item", parent_name),
                current_module_path,
                resource_base_path,
            );
            format!("{}[]", item_type)
        }
        schema::ParameterType::Object { properties } => {
            let properties = generate_parameter_type_properties(
                properties,
                decls,
                imports,
                parent_name,
                current_module_path,
                resource_base_path,
            );
            format!("{{{}}}", properties)
        }
        schema::ParameterType::EmptyObject => String::from("Record<string, never>"),
        schema::ParameterType::Enum { .. } => {
            format!("(typeof {parent_name}[keyof typeof {parent_name}] | string & {{}})")
        }
        schema::ParameterType::OneOf { properties } => {
            let type_path = resource_ref_to_path("../utils", resource_base_path);
            imports.insert(ImportEntry {
                type_name: "OneOfType".to_string(),
                path: type_path,
                is_type_only: true,
                alias: None,
            });

            let mut props = String::new();
            props.push_str("OneOfType<{");
            props.push_str(&generate_parameter_type_properties(
                properties,
                decls,
                imports,
                parent_name,
                current_module_path,
                resource_base_path,
            ));
            props.push_str("}>");
            props
        }
        schema::ParameterType::Union { types } => {
            let mut type_names = Vec::new();
            for (i, param) in types.iter().enumerate() {
                let type_name = generate_parameter(
                    param,
                    decls,
                    imports,
                    &format!("{}Union{}", parent_name, i),
                    current_module_path,
                    resource_base_path,
                );
                type_names.push(format!("({type_name})"));
            }
            type_names.join(" | ")
        }
        schema::ParameterType::Json => String::from("Record<string, any>"),
        schema::ParameterType::Intersection { types } => {
            let mut type_names = Vec::new();
            for (i, param) in types.iter().enumerate() {
                let type_name = generate_parameter(
                    param,
                    decls,
                    imports,
                    &format!("{}Intersection{}", parent_name, i),
                    current_module_path,
                    resource_base_path,
                );
                type_names.push(format!("({type_name})"));
            }
            type_names.join(" & ")
        }
        schema::ParameterType::DiscriminatedUnion {
            types,
            discriminator,
        } => {
            let mut variant_types = Vec::new();
            for (variant_name, variant_param) in types {
                let variant_type_name = format!(
                    "{parent_name}{variant_name}",
                    variant_name = variant_name.to_case(convert_case::Case::Pascal)
                );
                let variant_type = generate_parameter(
                    variant_param,
                    decls,
                    imports,
                    &variant_type_name,
                    current_module_path,
                    resource_base_path,
                );
                let discriminated_type = format!(
                    "({{ {discriminator}: '{variant_name}' }} & {{ {variant_description}{variant_name_camel}: {variant_type} }})",
                    variant_name_camel = variant_name.to_case(convert_case::Case::Camel),
                    variant_description = variant_param.description().to_jsdoc(false),
                );
                variant_types.push(discriminated_type);
            }
            variant_types.join(" | ")
        }
        schema::ParameterType::ResourceRef(resource) => {
            // 참조된 리소스의 타입 이름과 경로를 추출
            let resource_ref = resource.resource_ref();
            let type_name = RESOURCE_INDEX.with(|resource_index| {
                match resource_index.get(resource_ref).map(|r| r.name()) {
                    Some(Some(name)) => return name.to_string(),
                    _ => resource_ref.split('/').last().unwrap().to_string(),
                }
            });

            // 참조된 타입의 파일 경로 계산
            let type_path = resource_ref_to_path(resource_ref, resource_base_path);

            // ImportEntry 생성하여 imports에 추가
            imports.insert(ImportEntry {
                type_name: type_name.clone(),
                path: type_path,
                is_type_only: true,
                alias: None,
            });

            type_name
        }
        schema::ParameterType::Error {
            properties,
            transaction_type,
        } => {
            // 0. Generate PortOneError import
            let type_path = resource_ref_to_path("#/resources/exception/index", resource_base_path);
            imports.insert(ImportEntry {
                type_name: "PortOneError".to_string(),
                path: type_path.clone(),
                is_type_only: false,
                alias: None,
            });
            imports.insert(ImportEntry {
                type_name: "isPortOneError".to_string(),
                path: type_path,
                is_type_only: false,
                alias: None,
            });

            // 1. Generate type guards
            let type_guard_decl = ts_parse!(
                r#"
                function is{parent_name}(
                    error: unknown
                    ): error is {parent_name} {{
                    return (
                        isPortOneError(error) &&
                        error.__portOneErrorType === '{parent_name}'
                    )
                }}"# as JsFunctionDeclaration
            );
            decls.push(type_guard_decl.into());

            // 2. Generate error class
            let property_type_decls = generate_parameter_type_properties(
                properties,
                decls,
                imports,
                parent_name,
                current_module_path,
                resource_base_path,
            );
            let property_decls = properties
                .keys()
                .cloned()
                .collect::<Vec<String>>()
                .join(",");
            let property_assignments = properties
                .keys()
                .map(|property_name| format!("this.{property_name} = {property_name}"))
                .collect::<Vec<String>>()
                .join("\n");
            let class_decl = ts_parse!(
                r#"
                class {parent_name} extends Error implements PortOneError {{
                    static [Symbol.hasInstance](instance: unknown): boolean {{
                        return is{parent_name}(instance)
                    }}
                    __portOneErrorType = '{parent_name}'
                    {transaction_type}
                    {property_type_decls}

                    constructor({{
                        {property_decls}
                    }}: {{
                        {property_type_decls}
                    }}) {{
                        super(message)

                        {property_assignments}
                    }}
                }}"# as JsClassDeclaration,
                transaction_type = if let Some(transaction_type) = transaction_type {
                    format!("transactionType = '{transaction_type}'")
                } else {
                    String::new()
                },
            );
            decls.push(class_decl.into());

            // 3. Return the error class name
            parent_name.to_string()
        }
    }
}

fn generate_parameter_type_properties(
    properties: &IndexMap<String, schema::Parameter>,
    decls: &mut Vec<AnyJsDeclaration>,
    imports: &mut IndexSet<ImportEntry>,
    parent_name: &str,
    current_module_path: &PathBuf,
    resource_base_path: &PathBuf,
) -> String {
    let mut props = String::new();
    for (property_name, parameter) in properties {
        let prop_type = generate_parameter_type_property(
            property_name,
            decls,
            imports,
            parameter,
            parent_name,
            current_module_path,
            resource_base_path,
        );
        props.push_str(&prop_type);
        props.push('\n');
    }
    props
}

fn generate_parameter_type_property(
    property_name: &str,
    decls: &mut Vec<AnyJsDeclaration>,
    imports: &mut IndexSet<ImportEntry>,
    parameter: &schema::Parameter,
    parent_name: &str,
    current_module_path: &PathBuf,
    resource_base_path: &PathBuf,
) -> String {
    let parent_name = format!(
        "{parent_name}{property_name}",
        property_name = property_name.to_case(convert_case::Case::Pascal)
    );
    let member_type = generate_parameter(
        parameter,
        decls,
        imports,
        &parent_name,
        current_module_path,
        resource_base_path,
    );
    let description = parameter.description().to_jsdoc(parameter.deprecated());
    let optional_marker = if parameter.optional() { "?" } else { "" };
    let property_name = if property_name.contains('-') {
        format!("'{}'", property_name)
    } else {
        property_name.to_string()
    };
    format!("{description}{property_name}{optional_marker}: {member_type}")
}

fn generate_const_enum_declaration(
    name: &str,
    description: Option<&str>,
    parameter: &schema::ParameterType,
) -> Option<JsVariableDeclaration> {
    if let schema::ParameterType::Enum {
        variants,
        value_prefix,
    } = &parameter
    {
        let variants =
            variants
                .iter()
                .fold(String::new(), |mut output, (variant_name, variant)| {
                    let description = variant.description.to_jsdoc(false);
                    let value_prefix = value_prefix
                        .as_ref()
                        .map_or_else(String::new, |p| format!("{}_", p));
                    writeln!(
                        output,
                        "{description}'{variant_name}': '{value_prefix}{variant_name}',"
                    )
                    .unwrap();
                    output
                });
        let description = description.to_jsdoc(false);
        Some(ts_parse!(
            "{description}const {name} = {{{variants}}} as const" as JsVariableDeclaration
        ))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use crate::node_text;

    use super::*;
    use biome_js_syntax::AnyJsDeclaration;
    use biome_rowan::AstNode;
    use browser_sdk_schema::{self as schema};
    use indexmap::IndexMap;
    use indoc::indoc;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_generate_named_parameter() {
        let mut decls = Vec::new();
        let named_param = schema::Parameter::new(
            Some("UserId".to_string()),
            Some("The ID of the user".to_string()),
            schema::ParameterType::Integer,
            false,
            None,
            false,
        );

        let type_name = generate_named_parameter(
            &named_param,
            &mut decls,
            &mut IndexSet::new(),
            "",
            &PathBuf::new(),
            &PathBuf::new(),
        );
        assert_eq!(type_name, "UserId");

        // Check that the declaration was added
        assert_eq!(decls.len(), 1);

        // Check the generated type alias
        if let AnyJsDeclaration::TsTypeAliasDeclaration(type_alias) = &decls[0] {
            assert_eq!(
                node_text!(type_alias).to_string().trim(),
                indoc! {r#"
                    /**
                    * The ID of the user
                    */
                    type UserId = number;"#
                }
            );
        } else {
            panic!("Expected TsTypeAliasDeclaration");
        }
    }

    #[test]
    fn test_generate_unnamed_parameter_optional() {
        let mut decls = Vec::new();
        let unnamed_param = schema::Parameter::new(
            None,
            Some("Optional user name".to_string()),
            schema::ParameterType::String,
            true,
            None,
            false,
        );

        let type_def = generate_unnamed_parameter(
            &unnamed_param,
            &mut decls,
            &mut IndexSet::new(),
            "UserName",
            &PathBuf::new(),
            &PathBuf::new(),
        );
        assert_eq!(type_def, "string | undefined");
        assert!(decls.is_empty());
    }

    #[test]
    fn test_generate_parameter_type_array() {
        let mut decls = Vec::new();
        let array_item = schema::Parameter::new(
            None,
            Some("Array of integers".to_string()),
            schema::ParameterType::Integer,
            false,
            None,
            false,
        );
        let array_param = schema::ParameterType::Array {
            items: Box::new(array_item),
        };

        let type_def = generate_parameter_type(
            &array_param,
            &mut decls,
            &mut IndexSet::new(),
            "IntegerArray",
            &PathBuf::new(),
            &PathBuf::new(),
        );
        assert_eq!(type_def, "number[]");
        assert!(decls.is_empty());
    }

    #[test]
    fn test_generate_parameter_type_object() {
        let mut decls = Vec::new();
        let mut properties = IndexMap::new();
        properties.insert(
            "id".to_string(),
            schema::Parameter::new(
                Some("Id".to_string()),
                Some("User ID".to_string()),
                schema::ParameterType::Integer,
                false,
                None,
                false,
            ),
        );
        properties.insert(
            "name".to_string(),
            schema::Parameter::new(
                Some("Name".to_string()),
                Some("User name".to_string()),
                schema::ParameterType::String,
                false,
                None,
                false,
            ),
        );

        let object_param = schema::ParameterType::Object { properties };

        let type_def = generate_parameter_type(
            &object_param,
            &mut decls,
            &mut IndexSet::new(),
            "User",
            &PathBuf::new(),
            &PathBuf::new(),
        );
        assert_eq!(
            type_def,
            indoc! {r#"
                {
                /**
                * User ID
                */
                id: Id

                /**
                * User name
                */
                name: Name
                }"#
            }
        );

        // Check that type declarations for "Id" and "Name" were added
        assert_eq!(decls.len(), 2);

        // Check the first type alias
        if let AnyJsDeclaration::TsTypeAliasDeclaration(type_alias) = &decls[0] {
            assert_eq!(
                node_text!(type_alias).to_string().trim(),
                indoc! {r#"
                    /**
                    * User ID
                    */
                    type Id = number;"#
                }
            );
        } else {
            panic!("Expected TsTypeAliasDeclaration for 'Id'");
        }

        // Check the second type alias
        if let AnyJsDeclaration::TsTypeAliasDeclaration(type_alias) = &decls[1] {
            assert_eq!(
                node_text!(type_alias).to_string().trim(),
                indoc! {r#"
                    /**
                    * User name
                    */
                    type Name = string;"#
                }
            );
        } else {
            panic!("Expected TsTypeAliasDeclaration for 'Name'");
        }
    }

    #[test]
    fn test_generate_parameter_enum() {
        let mut decls = Vec::new();
        let mut variants = IndexMap::new();
        variants.insert(
            "Admin".to_string(),
            schema::EnumVariant {
                description: Some("Administrator".to_string()),
            },
        );
        variants.insert(
            "User".to_string(),
            schema::EnumVariant {
                description: Some("Regular user".to_string()),
            },
        );
        variants.insert(
            "Guest".to_string(),
            schema::EnumVariant {
                description: Some("Guest user".to_string()),
            },
        );

        let enum_param = schema::ParameterType::Enum {
            variants,
            value_prefix: None,
        };

        let type_def = generate_parameter(
            &schema::Parameter::new(
                Some("UserRole".to_string()),
                Some("User role".to_string()),
                enum_param,
                false,
                None,
                false,
            ),
            &mut decls,
            &mut IndexSet::new(),
            "",
            &PathBuf::new(),
            &PathBuf::new(),
        );
        assert_eq!(type_def, "UserRole");

        // Check that the enum declaration was added
        assert_eq!(decls.len(), 2);

        if let AnyJsDeclaration::JsVariableDeclaration(var_decl) = &decls[0] {
            assert_eq!(
                node_text!(var_decl).to_string().trim(),
                indoc! {r#"
                    /**
                    * User role
                    */
                    const UserRole = {
                    /**
                    * Administrator
                    */
                    'Admin': 'Admin',

                    /**
                    * Regular user
                    */
                    'User': 'User',

                    /**
                    * Guest user
                    */
                    'Guest': 'Guest',
                    } as const"#
                }
            );
        } else {
            panic!("Expected JsVariableDeclaration for 'UserRole'");
        }
        if let AnyJsDeclaration::TsTypeAliasDeclaration(type_alias) = &decls[1] {
            assert_eq!(
                node_text!(type_alias).to_string().trim(),
                indoc! {r#"
                    /**
                    * User role
                    */
                    type UserRole = (typeof UserRole[keyof typeof UserRole] | string & {});"#
                }
            );
        } else {
            panic!("Expected TsTypeAliasDeclaration for 'UserRole'");
        }
    }

    #[test]
    fn test_generate_parameter_type_oneof() {
        let mut decls = Vec::new();
        let mut properties = IndexMap::new();

        properties.insert(
            "TypeA".to_string(),
            schema::Parameter::new(
                None,
                Some("Type A".to_string()),
                schema::ParameterType::StringLiteral {
                    value: "A".to_string(),
                },
                false,
                None,
                false,
            ),
        );

        properties.insert(
            "TypeB".to_string(),
            schema::Parameter::new(
                None,
                Some("Type B".to_string()),
                schema::ParameterType::StringLiteral {
                    value: "B".to_string(),
                },
                false,
                None,
                false,
            ),
        );

        let oneof_param = schema::ParameterType::OneOf { properties };

        let type_def = generate_parameter_type(
            &oneof_param,
            &mut decls,
            &mut IndexSet::new(),
            "MyOneOf",
            &PathBuf::new(),
            &PathBuf::new(),
        );
        assert_eq!(
            type_def,
            indoc! {r#"
                OneOfType<{
                /**
                * Type A
                */
                TypeA: 'A'

                /**
                * Type B
                */
                TypeB: 'B'
                }>"#
            }
        );
    }

    #[test]
    fn test_generate_parameter_type_union() {
        let mut decls = Vec::new();

        let union_param = schema::ParameterType::Union {
            types: vec![
                schema::Parameter::new(
                    None,
                    Some("Type A".to_string()),
                    schema::ParameterType::StringLiteral {
                        value: "A".to_string(),
                    },
                    false,
                    None,
                    false,
                ),
                schema::Parameter::new(
                    None,
                    Some("Type B".to_string()),
                    schema::ParameterType::StringLiteral {
                        value: "B".to_string(),
                    },
                    false,
                    None,
                    false,
                ),
            ],
        };

        let type_def = generate_parameter_type(
            &union_param,
            &mut decls,
            &mut IndexSet::new(),
            "MyUnion",
            &PathBuf::new(),
            &PathBuf::new(),
        );
        assert_eq!(type_def, "('A') | ('B')");
    }

    #[test]
    fn test_generate_parameter_type_discriminated_union() {
        let mut decls = Vec::new();

        let mut types = IndexMap::new();

        types.insert(
            "Circle".to_string(),
            schema::Parameter::new(
                None,
                Some("Circle shape".to_string()),
                schema::ParameterType::Object {
                    properties: {
                        let mut props = IndexMap::new();
                        props.insert(
                            "radius".to_string(),
                            schema::Parameter::new(
                                None,
                                Some("Radius of the circle".to_string()),
                                schema::ParameterType::Integer,
                                false,
                                None,
                                false,
                            ),
                        );
                        props
                    },
                },
                false,
                None,
                false,
            ),
        );

        types.insert(
            "Rectangle".to_string(),
            schema::Parameter::new(
                None,
                Some("Rectangle shape".to_string()),
                schema::ParameterType::Object {
                    properties: {
                        let mut props = IndexMap::new();
                        props.insert(
                            "width".to_string(),
                            schema::Parameter::new(
                                None,
                                Some("Width of the rectangle".to_string()),
                                schema::ParameterType::Integer,
                                false,
                                None,
                                false,
                            ),
                        );
                        props.insert(
                            "height".to_string(),
                            schema::Parameter::new(
                                None,
                                Some("Height of the rectangle".to_string()),
                                schema::ParameterType::Integer,
                                false,
                                None,
                                false,
                            ),
                        );
                        props
                    },
                },
                false,
                None,
                false,
            ),
        );

        let discriminated_union_param = schema::ParameterType::DiscriminatedUnion {
            types,
            discriminator: "type".to_string(),
        };

        assert!(decls.is_empty());

        let type_def = generate_parameter_type(
            &discriminated_union_param,
            &mut decls,
            &mut IndexSet::new(),
            "Shape",
            &PathBuf::new(),
            &PathBuf::new(),
        );
        assert_eq!(
            type_def,
            indoc! {r#"
                ({ type: 'Circle' } & { 
                /**
                * Circle shape
                */
                circle: {
                /**
                * Radius of the circle
                */
                radius: number
                } }) | ({ type: 'Rectangle' } & { 
                /**
                * Rectangle shape
                */
                rectangle: {
                /**
                * Width of the rectangle
                */
                width: number

                /**
                * Height of the rectangle
                */
                height: number
                } })"#
            }
        );
    }
}
