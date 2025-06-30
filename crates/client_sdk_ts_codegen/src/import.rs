use std::path::{Path, PathBuf};

use biome_js_syntax::JsImport;
use client_sdk_ts_codegen_macros::ts_parse;
use indexmap::IndexSet;
use pathdiff::diff_paths;

pub(crate) struct ImportEntry {
    pub(crate) type_name: String,
    pub(crate) path: PathBuf,
    pub(crate) is_type_only: bool,
    pub(crate) alias: Option<String>,
}

impl PartialEq for ImportEntry {
    fn eq(&self, other: &Self) -> bool {
        self.type_name == other.type_name && self.path == other.path
    }
}

impl Eq for ImportEntry {}

impl std::hash::Hash for ImportEntry {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.type_name.hash(state);
        self.path.hash(state);
    }
}

pub(crate) fn resource_ref_to_path(resource_ref: &str, resource_base_path: &Path) -> PathBuf {
    // resource_ref는 "#/resources/..." 형식입니다.
    let resource_path = resource_ref.trim_start_matches("#/resources/");

    // 리소스 경로를 파일 경로로 변환
    let path_parts = resource_path.split('/').collect::<Vec<_>>();
    let mut path = resource_base_path.to_path_buf();
    for part in &path_parts {
        path = path.join(part);
    }

    // 파일 확장자 추가 (예: .ts)
    path.set_extension("ts");

    path
}

pub(crate) fn generate_import_statements(
    imports: &IndexSet<ImportEntry>,
    current_module_path: &Path,
) -> Vec<JsImport> {
    imports
        .iter()
        .map(|entry| {
            // 현재 모듈에서 참조된 타입의 경로까지의 상대 경로 계산
            let relative_path = diff_paths(&entry.path, current_module_path.parent().unwrap())
                .unwrap_or_else(|| entry.path.clone());

            // TypeScript에서 상대 경로는 '/' 대신 '\'를 사용할 수 있으므로, 이를 변환
            let relative_path = relative_path
                .to_str()
                .unwrap()
                .replace('\\', "/")
                // 파일 확장자를 '.js'로 변경
                .replace(".ts", ".js");

            // ../로 시작하지 않는 상대 경로는 ./로 시작하도록 변경
            let relative_path = if !relative_path.starts_with("../") {
                format!("./{relative_path}")
            } else {
                relative_path
            };

            match (entry.is_type_only, entry.alias.as_ref()) {
                (true, Some(alias)) => ts_parse!(
                    "import type {{ {} as {} }} from '{}';" as JsImport,
                    entry.type_name,
                    alias,
                    relative_path
                ),
                (false, Some(alias)) => ts_parse!(
                    "import {{ {} as {} }} from '{}';" as JsImport,
                    entry.type_name,
                    alias,
                    relative_path
                ),
                (true, None) => ts_parse!(
                    "import type {{ {} }} from '{}';" as JsImport,
                    entry.type_name,
                    relative_path
                ),
                (false, None) => ts_parse!(
                    "import {{ {} }} from '{}';" as JsImport,
                    entry.type_name,
                    relative_path
                ),
            }
        })
        .collect()
}
