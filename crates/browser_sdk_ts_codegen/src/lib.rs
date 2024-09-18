pub mod comment;
pub mod entrypoint;
pub mod import;
pub mod loader;
pub mod macros;
pub mod method;
pub mod parameter;
pub mod print;

use std::{fs, path::PathBuf};

use biome_js_factory::make::{self};
use biome_js_syntax::{AnyJsDeclaration, AnyJsModuleItem};
use biome_rowan::AstNode;

use browser_sdk_schema as schema;
use browser_sdk_ts_codegen_macros::ts_parse;
use import::{generate_import_statements, ImportEntry};
use indexmap::IndexSet;
use parameter::generate_named_parameter;

pub fn generate_resource_module(
    path: &PathBuf,
    resource_name: &str,
    resource: &schema::Resource,
    resource_base_path: &PathBuf, // 리소스의 기본 경로
) {
    match resource {
        schema::Resource::SubResources(index_map) => {
            let current_path = path.join(resource_name);
            fs::create_dir_all(&current_path).unwrap();

            let mut submodule_names = Vec::new();
            let mut subdirectories = Vec::new();

            for (name, resource) in index_map {
                generate_resource_module(&current_path, name, resource, resource_base_path);

                match resource {
                    schema::Resource::SubResources(_) => {
                        subdirectories.push(name.clone());
                    }
                    schema::Resource::Parameter(_) => {
                        submodule_names.push(name.clone());
                    }
                }
            }

            // Generate index.ts for the current directory
            if !resource_name.is_empty() {
                let mut index_ts_content = generate_index_ts(&submodule_names, &subdirectories);
                if resource_name == "exception" {
                    index_ts_content = print::print_node(
                        &ts_parse!(
                            r#"
                            export interface PortOneError extends Error {{
                                __portOneErrorType: string
                            }}
    
                            export function isPortOneError(error: unknown): error is PortOneError {{
                                return (
                                    error != null &&
                                    typeof error === 'object' &&
                                    '__portOneErrorType' in error &&
                                    typeof error.__portOneErrorType === 'string'
                                )
                            }}
                            
                            {index_ts_content}"# as JsModule
                        )
                        .into(),
                    );
                }
                fs::write(current_path.join("index.ts"), index_ts_content).unwrap();
            }
        }
        schema::Resource::Parameter(parameter) => {
            let current_module_path = path.join(format!("{}.ts", resource_name));
            let mut decls: Vec<AnyJsDeclaration> = Vec::new();
            let mut imports: IndexSet<ImportEntry> = IndexSet::new();
            let parameter = match parameter {
                schema::Parameter::Named(named_parameter) => named_parameter,
                schema::Parameter::Unnamed(unnamed_parameter) => &schema::NamedParameter::new(
                    resource_name.to_string(),
                    unnamed_parameter.clone(),
                ),
            };
            generate_named_parameter(
                parameter,
                &mut decls,
                &mut imports,
                "",
                &current_module_path,
                resource_base_path,
            );

            let import_statements = generate_import_statements(&imports, &current_module_path);
            let imports = import_statements
                .into_iter()
                .map(AnyJsModuleItem::from)
                .collect::<Vec<_>>();
            let module_items = imports
                .into_iter()
                .chain(
                    decls
                        .into_iter()
                        .map(|d| js_export!(d))
                        .map(AnyJsModuleItem::from),
                )
                .collect::<Vec<_>>();

            let module = make::js_module(
                make::js_directive_list([]),
                make::js_module_item_list(module_items),
                make::eof(),
            )
            .build();
            let module = print::print_node(&module.into());
            fs::create_dir_all(path).unwrap();
            fs::write(&current_module_path, module).unwrap();
        }
    };
}

fn generate_index_ts(submodule_names: &[String], subdirectories: &[String]) -> String {
    let mut exports = Vec::new();

    // Export modules (e.g., export * from './foo';)
    for module_name in submodule_names {
        exports.push(format!("export * from './{}.js';", module_name));
    }

    // Export subdirectories as namespaces (e.g., export * as b from './b';)
    for dir_name in subdirectories {
        exports.push(format!(
            "export * as {} from './{}/index.js';",
            dir_name, dir_name
        ));
    }

    exports.join("\n")
}
