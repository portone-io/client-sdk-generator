use std::{collections::HashMap, path::Path};

use ast::{
    Comment, CompositeType, Enum, EnumVariant, Identifier, Intersection, IntersectionConstituent,
    Object, ObjectField, ScalarType, TypeReference, Union, UnionParent, UnionVariant,
};
use client_sdk_schema::{Parameter, ParameterType, RESOURCE_INDEX, Resource, ResourceRef};
use client_sdk_utils::write_generated_file;
use convert_case::{Case, Casing};

pub mod ast;

enum Entity {
    Object(Object),
    Enum(Enum),
    Union(Union),
    Intersection(Intersection),
}

impl Entity {
    fn type_name(&self) -> &str {
        match self {
            Entity::Object(o) => o.name.as_ref(),
            Entity::Enum(e) => e.name.as_ref(),
            Entity::Union(u) => u.name.as_ref(),
            Entity::Intersection(i) => i.name.as_ref(),
        }
    }

    fn set_name(&mut self, new_name: &str) {
        let name = Identifier::try_from(new_name).unwrap();
        match self {
            Entity::Object(o) => o.name = name,
            Entity::Enum(e) => e.name = name,
            Entity::Union(u) => u.name = name,
            Entity::Intersection(i) => i.name = name,
        }
    }
}

struct ResourceProcessor {
    entities: HashMap<String, Entity>,
}

impl ResourceProcessor {
    fn resource_ref_to_type_reference(resource_ref: &ResourceRef) -> TypeReference {
        let path = resource_ref.resource_ref();
        RESOURCE_INDEX.with(|index| {
            let parameter = index.get(path).unwrap();
            let mut name = parameter
                .name
                .clone()
                .unwrap_or_else(|| path.rsplit_once('/').unwrap().1.to_string());
            // Swift Foundation의 Locale과 이름 충돌 방지
            if name == "Locale" {
                name = "PortOneLocale".to_string();
            }
            TypeReference {
                name: Identifier::try_from(name).unwrap(),
                path: path.into(),
            }
        })
    }

    fn collect_fields_from_type(&self, type_ref: &TypeReference) -> Vec<ObjectField> {
        RESOURCE_INDEX.with(|index| {
            if let Some(parameter) = index.get(&type_ref.path) {
                match &parameter.r#type {
                    ParameterType::Object { properties, .. } => {
                        Self::build_field_list(properties.iter())
                    }
                    _ => vec![],
                }
            } else {
                vec![]
            }
        })
    }

    fn build_field(name: &str, parameter: &Parameter) -> ObjectField {
        let field_name: Identifier = name.to_case(Case::Camel).try_into().unwrap();
        let is_required = !parameter.optional;
        let value_type = match &parameter.r#type {
            ParameterType::String | ParameterType::StringLiteral { .. } => CompositeType {
                scalar: ScalarType::String,
                is_list: false,
                is_required,
            },
            ParameterType::Integer => CompositeType {
                scalar: ScalarType::Int,
                is_list: false,
                is_required,
            },
            ParameterType::Boolean => CompositeType {
                scalar: ScalarType::Bool,
                is_list: false,
                is_required,
            },
            ParameterType::Json => CompositeType {
                scalar: ScalarType::Json,
                is_list: false,
                is_required,
            },
            ParameterType::Enum { .. } => CompositeType {
                scalar: ScalarType::String,
                is_list: false,
                is_required,
            },
            ParameterType::Array {
                items,
                hide_if_empty: _,
            } => {
                let scalar = match &items.r#type {
                    ParameterType::String | ParameterType::StringLiteral { .. } => {
                        ScalarType::String
                    }
                    ParameterType::Integer => ScalarType::Int,
                    ParameterType::Boolean => ScalarType::Bool,
                    ParameterType::ResourceRef(resource_ref) => ScalarType::TypeReference(
                        Self::resource_ref_to_type_reference(resource_ref),
                    ),
                    ParameterType::Json => ScalarType::Json,
                    ParameterType::Enum { .. } => ScalarType::String,
                    _ => unreachable!(),
                };
                CompositeType {
                    scalar,
                    is_list: true,
                    is_required,
                }
            }
            ParameterType::ResourceRef(resource_ref) => {
                return RESOURCE_INDEX.with(|index| {
                    let mut resource_ref = resource_ref;
                    loop {
                        let parameter = index.get(resource_ref.resource_ref()).unwrap();
                        match &parameter.r#type {
                            ParameterType::Object { .. }
                            | ParameterType::EmptyObject
                            | ParameterType::Enum { .. }
                            | ParameterType::Union { .. }
                            | ParameterType::OneOf { .. }
                            | ParameterType::Intersection { .. } => {
                                let value_type = CompositeType {
                                    scalar: ScalarType::TypeReference(
                                        Self::resource_ref_to_type_reference(resource_ref),
                                    ),
                                    is_list: false,
                                    is_required,
                                };
                                break ObjectField {
                                    name: field_name,
                                    serialized_name: name.to_string(),
                                    value_type,
                                    description: parameter
                                        .description
                                        .clone()
                                        .map(|d| Comment::try_from(d).unwrap()),
                                };
                            }
                            ParameterType::ResourceRef(r) => {
                                resource_ref = r;
                            }
                            _ => {
                                let mut field = Self::build_field(name, parameter);
                                field.value_type.is_required = is_required;
                                break field;
                            }
                        }
                    }
                });
            }
            _ => unreachable!("{:#?}", parameter.r#type),
        };
        ObjectField {
            name: field_name,
            serialized_name: name.to_string(),
            value_type,
            description: Self::build_field_description(parameter),
        }
    }

    fn build_field_description(parameter: &Parameter) -> Option<Comment> {
        let mut desc_parts = Vec::new();

        if let Some(base_desc) = &parameter.description {
            desc_parts.push(base_desc.clone());
        }

        if let ParameterType::Enum { variants, .. } = &parameter.r#type {
            let variant_lines: Vec<String> = variants
                .iter()
                .map(|(value, variant)| {
                    if let Some(variant_desc) = &variant.description {
                        format!("- `{value}`: {variant_desc}")
                    } else {
                        format!("- `{value}`")
                    }
                })
                .collect();

            if !variant_lines.is_empty() {
                desc_parts.push(variant_lines.join("\n"));
            }
        }

        if desc_parts.is_empty() {
            None
        } else {
            Some(Comment::try_from(desc_parts.join("\n\n")).unwrap())
        }
    }

    fn build_field_list<'a>(
        parameters: impl Iterator<Item = (&'a String, &'a Parameter)>,
    ) -> Vec<ObjectField> {
        let mut fields = vec![];
        for (name, parameter) in parameters {
            fields.push(Self::build_field(name, parameter));
        }
        fields
    }

    fn process_resource(&mut self, resource: &Resource, path: &mut Vec<String>) {
        match resource {
            Resource::SubResources(index_map) => {
                for (key, value) in index_map.iter() {
                    path.push(key.clone());
                    self.process_resource(value, path);
                    path.pop();
                }
            }
            Resource::Parameter(parameter) => {
                let name = path.last().unwrap();
                let path = path.join("/");
                if let Some(entity) = self.process_parameter(name, parameter) {
                    self.entities.insert(path, entity);
                }
            }
        }
    }

    fn connect_union_parents(&mut self) {
        let mut union_parents = HashMap::new();
        for (path, entity) in self.entities.iter_mut() {
            if let Entity::Union(parent) = entity {
                let parent_ref = TypeReference {
                    path: path.clone(),
                    name: parent.name.clone(),
                };
                for variant in parent.variants.iter() {
                    union_parents
                        .entry(variant.type_name.path.clone())
                        .or_insert(vec![])
                        .push(UnionParent::Union {
                            parent: parent_ref.clone(),
                            variant_name: variant.name.clone(),
                        });
                }
            }
        }
        for (path, parents) in union_parents {
            match self.entities.get_mut(&path).unwrap() {
                Entity::Object(Object { union_parents, .. })
                | Entity::Intersection(Intersection { union_parents, .. })
                | Entity::Enum(Enum { union_parents, .. }) => {
                    *union_parents = parents;
                }
                _ => unreachable!(),
            }
        }
    }

    fn process_parameter(&mut self, name: &str, parameter: &Parameter) -> Option<Entity> {
        let raw_name = parameter.name.as_deref().unwrap_or(name);
        // Swift Foundation의 Locale과 이름 충돌 방지
        let raw_name = if raw_name == "Locale" {
            "PortOneLocale"
        } else {
            raw_name
        };
        let name: Identifier = raw_name.try_into().unwrap();
        match &parameter.r#type {
            ParameterType::Object {
                properties,
                hide_if_empty: _,
            } => Some(Entity::Object(Object {
                name: name.clone(),
                description: parameter
                    .description
                    .clone()
                    .map(|d| Comment::try_from(d).unwrap()),
                fields: Self::build_field_list(properties.iter()),
                is_one_of: false,
                union_parents: vec![],
            })),
            ParameterType::EmptyObject => Some(Entity::Object(Object {
                name: name.clone(),
                description: parameter
                    .description
                    .clone()
                    .map(|d| Comment::try_from(d).unwrap()),
                fields: vec![],
                is_one_of: false,
                union_parents: vec![],
            })),
            ParameterType::Enum { variants, .. } => Some(Entity::Enum(Enum {
                name: name.clone(),
                description: parameter
                    .description
                    .clone()
                    .map(|d| Comment::try_from(d).unwrap()),
                variants: variants
                    .iter()
                    .map(|(value, variant)| EnumVariant {
                        name: if let Some(alias) = &variant.alias {
                            Identifier::try_from(alias.as_str()).unwrap()
                        } else {
                            Identifier::try_from(value.as_str()).unwrap()
                        },
                        value: value.clone(),
                        description: variant
                            .description
                            .clone()
                            .map(|d| Comment::try_from(d).unwrap()),
                    })
                    .collect(),
                union_parents: vec![],
            })),
            ParameterType::OneOf {
                properties,
                hide_if_empty: _,
            } => Some(Entity::Object(Object {
                name: name.clone(),
                description: parameter
                    .description
                    .clone()
                    .map(|d| Comment::try_from(d).unwrap()),
                fields: Self::build_field_list(properties.iter()),
                is_one_of: true,
                union_parents: vec![],
            })),
            ParameterType::Union {
                types,
                hide_if_empty: _,
            } => Some(Entity::Union(Union {
                name: name.clone(),
                description: parameter
                    .description
                    .clone()
                    .map(|d| Comment::try_from(d).unwrap()),
                variants: types
                    .iter()
                    .map(|parameter| match &parameter.r#type {
                        ParameterType::ResourceRef(resource_ref) => {
                            let type_reference = Self::resource_ref_to_type_reference(resource_ref);
                            UnionVariant {
                                name: type_reference.name.as_ref().try_into().unwrap(),
                                description: parameter
                                    .description
                                    .clone()
                                    .map(|d| Comment::try_from(d).unwrap()),
                                type_name: type_reference,
                            }
                        }
                        _ => unreachable!(),
                    })
                    .collect(),
            })),
            ParameterType::Intersection {
                types,
                hide_if_empty: _,
            } => {
                let constituents: Vec<IntersectionConstituent> = types
                    .iter()
                    .map(|parameter| match &parameter.r#type {
                        ParameterType::ResourceRef(resource_ref) => {
                            let type_reference = Self::resource_ref_to_type_reference(resource_ref);
                            IntersectionConstituent {
                                name: type_reference.name.as_ref().to_string().try_into().unwrap(),
                                type_name: type_reference,
                            }
                        }
                        _ => unreachable!(),
                    })
                    .collect();

                // Collect all fields from constituents
                let mut all_fields = Vec::new();
                let mut field_names = std::collections::HashSet::new();

                for constituent in &constituents {
                    let fields = self.collect_fields_from_type(&constituent.type_name);
                    for field in fields {
                        // If field name already exists, replace it (later constituent wins)
                        if field_names.contains(field.name.as_ref()) {
                            all_fields
                                .retain(|f: &ObjectField| f.name.as_ref() != field.name.as_ref());
                        }
                        field_names.insert(field.name.as_ref().to_string());
                        all_fields.push(field);
                    }
                }

                Some(Entity::Intersection(Intersection {
                    name: name.clone(),
                    description: parameter
                        .description
                        .clone()
                        .map(|d| Comment::try_from(d).unwrap()),
                    constituents,
                    fields: all_fields,
                    union_parents: vec![],
                }))
            }
            _ => None,
        }
    }

    fn extract_prefix_from_path(path: &str) -> String {
        // "entity/bypass/payment/EximbayV2BillTo" -> "Payment"
        // 타입명 직전 디렉토리를 접두사로 사용
        if let Some((parent_path, _type_name)) = path.rsplit_once('/')
            && let Some((_, parent_dir)) = parent_path.rsplit_once('/')
        {
            return parent_dir.to_case(Case::Pascal);
        }
        String::new()
    }

    fn update_type_references(entity: &mut Entity, name_mappings: &HashMap<String, String>) {
        let update_field = |field: &mut ObjectField| {
            if let ScalarType::TypeReference(ref mut type_ref) = field.value_type.scalar
                && let Some(new_name) = name_mappings.get(&type_ref.path)
            {
                type_ref.name = Identifier::try_from(new_name.as_str()).unwrap();
            }
        };

        match entity {
            Entity::Object(obj) => {
                for field in &mut obj.fields {
                    update_field(field);
                }
            }
            Entity::Intersection(inter) => {
                for field in &mut inter.fields {
                    update_field(field);
                }
            }
            Entity::Union(union) => {
                for variant in &mut union.variants {
                    if let Some(new_name) = name_mappings.get(&variant.type_name.path) {
                        variant.type_name.name = Identifier::try_from(new_name.as_str()).unwrap();
                        variant.name = Identifier::try_from(new_name.as_str()).unwrap();
                    }
                }
            }
            Entity::Enum(_) => {}
        }
    }

    fn generate_directory(mut self, file_base_path: impl AsRef<Path>, _module_name: &str) {
        let file_base_path = file_base_path.as_ref();

        // Collect paths that are intersection constituents
        let constituent_paths: std::collections::HashSet<String> = self
            .entities
            .values()
            .filter_map(|entity| {
                if let Entity::Intersection(intersection) = entity {
                    Some(
                        intersection
                            .constituents
                            .iter()
                            .map(|c| c.type_name.path.clone()),
                    )
                } else {
                    None
                }
            })
            .flatten()
            .collect();

        // Count type names to detect duplicates
        let mut type_name_counts: HashMap<String, usize> = HashMap::new();
        for (path, entity) in &self.entities {
            if constituent_paths.contains(path) {
                continue;
            }
            *type_name_counts
                .entry(entity.type_name().to_string())
                .or_default() += 1;
        }

        // Build name mappings for duplicate types (path -> new_name)
        let mut name_mappings: HashMap<String, String> = HashMap::new();
        for (path, entity) in &self.entities {
            let original_name = entity.type_name().to_string();
            if type_name_counts.get(&original_name).copied().unwrap_or(0) > 1 {
                let prefix = Self::extract_prefix_from_path(path);
                let new_name = format!("{}{}", prefix, original_name);
                name_mappings.insert(path.clone(), new_name);
            }
        }

        // Update all type references to use new names
        for entity in self.entities.values_mut() {
            Self::update_type_references(entity, &name_mappings);
        }

        for (path, mut entity) in self.entities {
            // Skip generating files for intersection constituents
            if constituent_paths.contains(&path) {
                continue;
            }

            let final_name = if let Some(new_name) = name_mappings.get(&path) {
                entity.set_name(new_name);
                new_name.clone()
            } else {
                entity.type_name().to_string()
            };

            let content = match entity {
                Entity::Object(object) => {
                    use std::fmt::Write;
                    let mut content = String::new();
                    writeln!(&mut content, "import Foundation").unwrap();
                    writeln!(content).unwrap();
                    write!(content, "{object}").unwrap();
                    content
                }
                Entity::Enum(enum_entity) => {
                    use std::fmt::Write;
                    let mut content = String::new();
                    writeln!(&mut content, "import Foundation").unwrap();
                    writeln!(content).unwrap();
                    write!(content, "{enum_entity}").unwrap();
                    content
                }
                Entity::Union(union) => {
                    use std::fmt::Write;
                    let mut content = String::new();
                    writeln!(&mut content, "import Foundation").unwrap();
                    writeln!(content).unwrap();
                    write!(content, "{union}").unwrap();
                    content
                }
                Entity::Intersection(intersection) => {
                    use std::fmt::Write;
                    let mut content = String::new();
                    writeln!(&mut content, "import Foundation").unwrap();
                    writeln!(content).unwrap();
                    write!(content, "{intersection}").unwrap();
                    content
                }
            };

            let mut file_path =
                file_base_path.join(path.rsplit_once('/').map(|(dir, _)| dir).unwrap_or(""));
            std::fs::create_dir_all(&file_path).unwrap();
            file_path = file_path.join(&final_name);
            file_path.set_extension("swift");
            write_generated_file(file_path, content).unwrap();
        }
    }
}

pub fn generate_resources_module(
    resource: &Resource,
    file_base_path: impl AsRef<Path>,
    module_name: &str,
) {
    let mut processor = ResourceProcessor {
        entities: HashMap::new(),
    };
    if let Resource::SubResources(subresources) = resource {
        for (key, value) in subresources.iter() {
            if matches!(key.as_str(), "entity" | "request" | "response") {
                processor.process_resource(value, &mut vec![key.clone()]);
            }
        }
    }
    // Mobile-only transformations (iOS specific)
    for (path, entity) in processor.entities.iter_mut() {
        if path.starts_with("request/") {
            if let Entity::Object(object) = entity {
                object
                    .fields
                    .retain(|field| field.name.as_ref() != "redirectUrl");
                for field in object.fields.iter_mut() {
                    if field.name.as_ref() == "appScheme" {
                        field.value_type.is_required = false;
                    }
                }
            }
            if let Entity::Intersection(intersection) = entity {
                intersection
                    .fields
                    .retain(|field| field.name.as_ref() != "redirectUrl");
                for field in intersection.fields.iter_mut() {
                    if field.name.as_ref() == "appScheme" {
                        field.value_type.is_required = false;
                    }
                }
            }
        }
    }
    processor.connect_union_parents();
    processor.generate_directory(&file_base_path, module_name);

    // JSONValue.swift 파일 생성
    let json_value_content = r#"import Foundation

public enum JSONValue: Codable, Equatable {
    case null
    case bool(Bool)
    case int(Int)
    case double(Double)
    case string(String)
    case array([JSONValue])
    case object([String: JSONValue])

    public init(from decoder: Decoder) throws {
        let container = try decoder.singleValueContainer()
        if container.decodeNil() {
            self = .null
        } else if let bool = try? container.decode(Bool.self) {
            self = .bool(bool)
        } else if let int = try? container.decode(Int.self) {
            self = .int(int)
        } else if let double = try? container.decode(Double.self) {
            self = .double(double)
        } else if let string = try? container.decode(String.self) {
            self = .string(string)
        } else if let array = try? container.decode([JSONValue].self) {
            self = .array(array)
        } else if let object = try? container.decode([String: JSONValue].self) {
            self = .object(object)
        } else {
            throw DecodingError.dataCorruptedError(in: container, debugDescription: "Invalid JSON value")
        }
    }

    public func encode(to encoder: Encoder) throws {
        var container = encoder.singleValueContainer()
        switch self {
        case .null: try container.encodeNil()
        case .bool(let v): try container.encode(v)
        case .int(let v): try container.encode(v)
        case .double(let v): try container.encode(v)
        case .string(let v): try container.encode(v)
        case .array(let v): try container.encode(v)
        case .object(let v): try container.encode(v)
        }
    }
}
"#;
    let json_value_path = file_base_path.as_ref().join("JSONValue.swift");
    write_generated_file(json_value_path, json_value_content.to_string()).unwrap();
}
