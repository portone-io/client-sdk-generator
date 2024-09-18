pub(crate) trait JsDocExt<T: AsRef<str>> {
    fn to_jsdoc(&self) -> String;
}

impl<T: AsRef<str>> JsDocExt<T> for T {
    fn to_jsdoc(&self) -> String {
        generate_jsdoc_comment(self.as_ref())
    }
}

impl<T: AsRef<str>> JsDocExt<T> for Option<T> {
    fn to_jsdoc(&self) -> String {
        match self {
            Some(comment) => generate_jsdoc_comment(comment.as_ref()),
            None => String::new(),
        }
    }
}

pub fn generate_jsdoc_comment(comment: &str) -> String {
    let comment = comment
        .trim()
        .split('\n')
        .map(|line| format!("* {}", line))
        .collect::<Vec<_>>()
        .join("\n");

    format!("\n/**\n{}\n*/\n", comment)
}
