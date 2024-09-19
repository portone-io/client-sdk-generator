use std::path::PathBuf;
use std::{fmt::Write, fs};

use browser_sdk_ts_codegen_macros::ts_parse;
use indexmap::IndexMap;

use browser_sdk_schema::{self as schema};

use crate::print;

pub fn generate_entrypoint_module(path: &PathBuf, methods: &IndexMap<String, schema::Method>) {
    let current_module_path = path.join("index.ts");
    let imports = methods.keys().fold(String::new(), |mut acc, method_name| {
        acc.push_str(
            &ts_parse!("import {{ {method_name} }} from './{method_name}.js';" as JsImport)
                .to_string(),
        );
        acc
    });

    let portone_object_decl = ts_parse!(
        r#"
        export const PortOne = {{
            {methods}
        }};
        "# as JsVariableDeclaration,
        methods = methods.keys().fold(String::new(), |mut acc, method_name| {
            writeln!(acc, "{method_name},").unwrap();
            acc
        }),
    )
    .to_string();

    let method_exports = methods.keys().fold(String::new(), |mut acc, method_name| {
        acc.push_str(&ts_parse!("export * from './{method_name}.js';" as JsExport).to_string());
        acc
    });
    let index_ts = fs::read_to_string(&current_module_path).unwrap();
    let module = ts_parse!(
        r#"
        {imports}

        {portone_object_decl}
        
        export {{ setPortOneJsSdkUrl as __INTERNAL__setPortOneSdkUrl }} from './loader.js'

        {method_exports}

        {index_ts}

        export * as Entity from './entity/index.js';
        export * as errors from './exception/index.js';

        export default PortOne;
        "# as JsModule,
    );
    let module = print::print_node(&module.into());
    fs::create_dir_all(path).unwrap();
    fs::write(&current_module_path, module).unwrap();
}
