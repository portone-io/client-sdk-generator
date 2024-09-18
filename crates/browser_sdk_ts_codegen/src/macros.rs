#[macro_export]
macro_rules! js_str {
    ($str_literal:expr) => {
        &format!(r#""{}""#, $str_literal)
    };
}

#[macro_export]
macro_rules! js_export {
    ($item:expr) => {{
        let syntax = $item.syntax();
        let leading_trivia = syntax.first_leading_trivia();
        let trailing_trivia = syntax.last_trailing_trivia();
        ts_parse!(
            "{}export {}{}" as JsExport,
            leading_trivia
                .as_ref()
                .map_or_else(|| "", |trivia| trivia.text()),
            $item.syntax().text_trimmed(),
            trailing_trivia
                .as_ref()
                .map_or_else(|| "", |trivia| trivia.text()),
        )
    }};
}

#[macro_export]
macro_rules! node_text {
    ($node:expr) => {
        $node.syntax().text()
    };
}

#[macro_export]
macro_rules! node_text_join {
    ($node_list:expr) => {
        $node_list.iter().fold(String::new(), |mut acc, node| {
            write!(acc, "{}", node_text!(node)).unwrap();
            acc
        })
    };
}
