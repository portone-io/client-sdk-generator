use std::fmt::Write;
use std::fs;
use std::path::PathBuf;

use client_sdk_schema::{self as schema};
use client_sdk_ts_codegen_macros::ts_parse;
use indexmap::{IndexMap, IndexSet};

use crate::import::generate_import_statements;
use crate::method::generate_callbacks;
use crate::parameter::generate_parameter;
use crate::{print, write_generated_ts_file};

pub fn generate_loader(path: &PathBuf, methods: &IndexMap<String, schema::Method>) {
    let loader = include_str!("../templates/loader.ts");

    let mut decls = Vec::new();
    let mut imports = IndexSet::new();

    let current_module_path = path.join("loader.ts");
    let methods = methods
        .iter()
        .fold(String::new(), |mut acc, (method_name, method)| {
            let input = generate_parameter(
                &method.input,
                &mut decls,
                &mut imports,
                "",
                &current_module_path,
                path,
            );
            let output = match &method.output {
                Some(output) => generate_parameter(
                    output,
                    &mut decls,
                    &mut imports,
                    "",
                    &current_module_path,
                    path,
                ),
                None => "void".to_string(),
            };
            let callbacks = generate_callbacks(
                method.callbacks.as_ref(),
                &mut decls,
                &mut imports,
                &current_module_path,
                path,
            );

            match &callbacks {
                Some(callbacks) => {
                    writeln!(
                        acc,
                        "{method_name}(request: {input}, callbacks: {{{callbacks}}}): Promise<{output}>",
                    )
                    .unwrap();
                }
                None => {
                    writeln!(
                        acc,
                        "{method_name}(request: {input}): Promise<{output}>",
                    )
                    .unwrap();
                }
            }
            acc
        });

    let interface = ts_parse!(
        r#"
        interface PortOne {{
            {methods}
        }}"# as TsInterfaceDeclaration,
    );

    let imports = generate_import_statements(&imports, &current_module_path)
        .into_iter()
        .fold(String::new(), |mut acc, import| {
            acc.push_str(&import.to_string());
            acc
        });
    let decls = decls.into_iter().fold(String::new(), |mut acc, decl| {
        acc.push_str(&decl.to_string());
        acc
    });
    let module = ts_parse!(
        r#"
        {imports}
        {decls}
        {interface}
        {loader}"# as JsModule,
    );
    let module = print::print_node(&module.into());
    fs::create_dir_all(path).unwrap();
    write_generated_ts_file(&current_module_path, module).unwrap();
}
