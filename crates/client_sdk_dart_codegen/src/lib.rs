use std::{collections::HashMap, path::Path};

use ast::{
    Comment, CompositeType, Enum, EnumVariant, Identifier, Intersection, IntersectionConstituent,
    Object, ObjectField, ScalarType, TypeReference, Union, UnionParent, UnionVariant,
    capitalize_first,
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

struct ResourceProcessor {
    entities: HashMap<String, Entity>,
}

impl ResourceProcessor {
    fn resource_ref_to_type_reference(resource_ref: &ResourceRef) -> TypeReference {
        let path = resource_ref.resource_ref();
        RESOURCE_INDEX.with(|index| {
            let parameter = index.get(path).unwrap();
            let name = parameter
                .name
                .clone()
                .unwrap_or_else(|| path.rsplit_once('/').unwrap().1.to_string());
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

    fn type_reference_to_import_path(
        type_reference: &TypeReference,
        import_base_path: &Path,
    ) -> String {
        let mut import_path = import_base_path.join(type_reference.path.to_case(Case::Snake));
        import_path.set_extension("dart");
        import_path.to_string_lossy().to_string()
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
                scalar: ScalarType::Object,
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
                    ParameterType::Json => ScalarType::Object,
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
                                    import_alias: None,
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
            import_alias: None,
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
        let name: Identifier = parameter
            .name
            .as_deref()
            .unwrap_or(name)
            .try_into()
            .unwrap();
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
                skip_from_json: false,
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
                skip_from_json: false,
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
                skip_from_json: false,
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
                                name: type_reference
                                    .name
                                    .as_ref()
                                    .to_case(Case::Camel)
                                    .try_into()
                                    .unwrap(),
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
                                name: type_reference
                                    .name
                                    .as_ref()
                                    .to_string()
                                    .to_case(Case::Camel)
                                    .try_into()
                                    .unwrap(),
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
                    skip_from_json: false,
                }))
            }
            _ => None,
        }
    }

    fn generate_directory(
        self,
        file_base_path: impl AsRef<Path>,
        import_base_path: impl AsRef<Path>,
    ) {
        let file_base_path = file_base_path.as_ref();
        let import_base_path = import_base_path.as_ref();

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

        for (path, entity) in self.entities {
            // Skip generating files for intersection constituents
            if constituent_paths.contains(&path) {
                continue;
            }

            let content = match entity {
                Entity::Object(mut object) => {
                    // response/ 가 아니면 fromJson 생략
                    object.skip_from_json = !path.starts_with("response/");

                    // OneOf 이름 충돌 감지
                    if object.is_one_of {
                        let parent_name = object.name.as_ref().to_string();
                        for field in &mut object.fields {
                            let subclass_name =
                                format!("{}{}", parent_name, capitalize_first(field.name.as_ref()));
                            if let ScalarType::TypeReference(type_ref) = &field.value_type.scalar
                                && type_ref.name.as_ref() == subclass_name
                            {
                                field.import_alias = Some(format!(
                                    "_{}",
                                    type_ref.name.as_ref().to_case(Case::Snake)
                                ));
                            }
                        }
                    }

                    // Build imports with alias support
                    let mut import_entries: Vec<(String, Option<String>)> = Vec::new();
                    for field in object.fields.iter() {
                        if let ScalarType::TypeReference(reference) = &field.value_type.scalar {
                            let import_path =
                                Self::type_reference_to_import_path(reference, import_base_path);
                            import_entries.push((import_path, field.import_alias.clone()));
                        }
                    }
                    for parent in object.union_parents.iter() {
                        let UnionParent::Union { parent, .. } = parent;
                        let import_path =
                            Self::type_reference_to_import_path(parent, import_base_path);
                        import_entries.push((import_path, None));
                    }
                    import_entries.sort_by(|a, b| a.0.cmp(&b.0));
                    import_entries.dedup_by(|a, b| a.0 == b.0);

                    use std::fmt::Write;
                    let mut content = String::new();
                    for (import_path, alias) in &import_entries {
                        if let Some(alias) = alias {
                            writeln!(&mut content, "import '{import_path}' as {alias};").unwrap();
                        } else {
                            writeln!(&mut content, "import '{import_path}';").unwrap();
                        }
                    }
                    writeln!(content).unwrap();
                    write!(content, "{object}").unwrap();
                    content
                }
                Entity::Enum(enum_entity) => {
                    let union_parents_refs = enum_entity
                        .union_parents
                        .iter()
                        .map(|UnionParent::Union { parent, .. }| parent);
                    let mut imports = union_parents_refs
                        .map(|reference| {
                            Self::type_reference_to_import_path(reference, import_base_path)
                        })
                        .collect::<Vec<_>>();
                    imports.sort();
                    imports.dedup();

                    use std::fmt::Write;
                    let mut content = String::new();
                    if !imports.is_empty() {
                        for import in imports {
                            writeln!(&mut content, "import '{import}';").unwrap();
                        }
                        writeln!(content).unwrap();
                    }
                    write!(content, "{enum_entity}").unwrap();
                    content
                }
                Entity::Union(union) => {
                    let variants_refs = union.variants.iter().map(|variant| &variant.type_name);
                    let mut imports = variants_refs
                        .map(|reference| {
                            Self::type_reference_to_import_path(reference, import_base_path)
                        })
                        .collect::<Vec<_>>();
                    imports.sort();
                    imports.dedup();

                    use std::fmt::Write;
                    let mut content = String::new();
                    for import in imports {
                        writeln!(&mut content, "import '{import}';").unwrap();
                    }
                    writeln!(content).unwrap();
                    write!(content, "{union}").unwrap();
                    content
                }

                Entity::Intersection(mut intersection) => {
                    intersection.skip_from_json = !path.starts_with("response/");
                    let fields_refs = intersection.fields.iter().flat_map(|field| {
                        if let ScalarType::TypeReference(reference) = &field.value_type.scalar {
                            Some(reference)
                        } else {
                            None
                        }
                    });
                    let union_parents_refs = intersection
                        .union_parents
                        .iter()
                        .map(|UnionParent::Union { parent, .. }| parent);
                    let mut imports = fields_refs
                        .chain(union_parents_refs)
                        .map(|reference| {
                            Self::type_reference_to_import_path(reference, import_base_path)
                        })
                        .collect::<Vec<_>>();
                    imports.sort();
                    imports.dedup();

                    use std::fmt::Write;
                    let mut content = String::new();
                    for import in imports {
                        writeln!(&mut content, "import '{import}';").unwrap();
                    }
                    writeln!(content).unwrap();
                    write!(content, "{intersection}").unwrap();
                    content
                }
            };
            let mut file_path = file_base_path.join(path.to_case(Case::Snake));
            file_path.set_extension("dart");
            std::fs::create_dir_all(file_path.parent().unwrap()).unwrap();
            write_generated_file(file_path, content).unwrap();
        }
    }
}

pub fn generate_resources_module(
    resource: &Resource,
    file_base_path: impl AsRef<Path>,
    import_base_path: impl AsRef<Path>,
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
    // Mobile-only transformations
    for (path, entity) in processor.entities.iter_mut() {
        if path.starts_with("request/") {
            let fields = match entity {
                Entity::Object(object) => &mut object.fields,
                Entity::Intersection(intersection) => &mut intersection.fields,
                _ => continue,
            };
            fields.retain(|field| field.name.as_ref() != "redirectUrl");
            for field in fields.iter_mut() {
                if field.name.as_ref() == "appScheme" {
                    field.value_type.is_required = true;
                }
            }
        }
    }
    processor.connect_union_parents();
    processor.generate_directory(file_base_path, import_base_path);
}
