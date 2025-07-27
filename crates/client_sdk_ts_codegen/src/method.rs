use std::fmt::Write;
use std::fs;
use std::path::PathBuf;

use biome_js_factory::make::{self};
use biome_js_syntax::{AnyJsDeclaration, AnyJsModuleItem};
use biome_rowan::AstNode;
use client_sdk_schema::{self as schema};
use client_sdk_ts_codegen_macros::ts_parse;
use indexmap::{IndexMap, IndexSet};

use crate::comment::JsDocExt;
use crate::import::{ImportEntry, generate_import_statements};
use crate::parameter::generate_parameter;
use crate::{js_export, print};
use client_sdk_utils::write_generated_file;

pub fn generate_method_modules(path: &PathBuf, methods: &IndexMap<String, schema::Method>) {
    for (method_name, method) in methods {
        generate_method_module(path, method_name, method, path);
    }
}

pub fn generate_method_module(
    path: &PathBuf,
    method_name: &str,
    method: &schema::Method,
    resource_base_path: &PathBuf, // 리소스의 기본 경로
) {
    let current_module_path = path.join(format!("{method_name}.ts"));

    let mut decls = Vec::new();
    let mut imports = IndexSet::new();
    imports.insert(ImportEntry {
        type_name: "loadScript".to_string(),
        path: resource_base_path.join("loader.js"),
        is_type_only: false,
        alias: None,
    });

    let input = generate_parameter(
        &method.input,
        &mut decls,
        &mut imports,
        "",
        &current_module_path,
        resource_base_path,
    );

    let callbacks = generate_callbacks(
        method.callbacks.as_ref(),
        &mut decls,
        &mut imports,
        &current_module_path,
        resource_base_path,
    );

    let output = match &method.output {
        Some(parameter) => generate_parameter(
            parameter,
            &mut decls,
            &mut imports,
            "",
            &current_module_path,
            resource_base_path,
        ),
        None => "void".to_string(),
    };

    let description = method.description.to_jsdoc(false);
    let func = match callbacks {
        Some(callbacks) => ts_parse!(
            r#"
            {description}function {method_name}(
                request: {input},
                callbacks: {{{callbacks}}},
            ): Promise<{output}> {{
                return loadScript().then((sdk) =>
                    sdk.{method_name}(request, callbacks)
                )
            }}
            "# as JsFunctionDeclaration,
        ),
        None => ts_parse!(
            r#"
            {description}function {method_name}(
                request: {input},
            ): Promise<{output}> {{
                return loadScript().then((sdk) =>
                    sdk.{method_name}(request)
                )
            }}
            "# as JsFunctionDeclaration,
        ),
    };

    let imports = generate_import_statements(&imports, &current_module_path)
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
        .chain(std::iter::once(AnyJsModuleItem::from(js_export!(func))))
        .collect::<Vec<_>>();

    let module_items = make::js_module_item_list(module_items);
    let module = make::js_module(make::js_directive_list([]), module_items, make::eof()).build();
    let module = print::print_node(&module.into());
    fs::create_dir_all(path).unwrap();
    write_generated_file(&current_module_path, module).unwrap();
}

pub(crate) fn generate_callbacks(
    callbacks: Option<&IndexMap<String, schema::Callback>>,
    decls: &mut Vec<AnyJsDeclaration>,
    imports: &mut IndexSet<ImportEntry>,
    current_module_path: &PathBuf,
    resource_base_path: &PathBuf,
) -> Option<String> {
    match callbacks {
        Some(callbacks) => {
            let callbacks =
                callbacks
                    .iter()
                    .fold(String::new(), |mut output, (callback_name, callback)| {
                        let description = callback.description.to_jsdoc(false);
                        let callback = callback.input.iter().fold(
                            String::new(),
                            |mut acc, (parameter_name, parameter)| {
                                write!(
                                    acc,
                                    "{parameter_name}: {},",
                                    generate_parameter(
                                        parameter,
                                        decls,
                                        imports,
                                        "",
                                        current_module_path,
                                        resource_base_path
                                    )
                                )
                                .unwrap();
                                acc
                            },
                        );
                        writeln!(output, "{description}{callback_name}: ({callback}) => void",)
                            .unwrap();
                        output
                    });
            Some(callbacks)
        }
        None => None,
    }
}
