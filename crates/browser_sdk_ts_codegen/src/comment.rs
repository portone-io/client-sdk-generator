pub(crate) trait JsDocExt<T: AsRef<str>> {
    fn to_jsdoc(&self, deprecated: bool) -> String;
}

impl<T: AsRef<str>> JsDocExt<T> for T {
    fn to_jsdoc(&self, deprecated: bool) -> String {
        generate_jsdoc_comment(self.as_ref(), deprecated)
    }
}

impl<T: AsRef<str>> JsDocExt<T> for Option<T> {
    fn to_jsdoc(&self, deprecated: bool) -> String {
        match self {
            Some(comment) => generate_jsdoc_comment(comment.as_ref(), deprecated),
            None => String::new(),
        }
    }
}

pub fn generate_jsdoc_comment(comment: &str, deprecated: bool) -> String {
    let mut comment = comment
        .trim()
        .split('\n')
        .map(|line| format!("* {}", line))
        .collect::<Vec<_>>();

    if deprecated {
        comment.push("* @deprecated".to_string());
    }

    let comment = comment.join("\n");

    format!("\n/**\n{}\n*/\n", comment)
}
