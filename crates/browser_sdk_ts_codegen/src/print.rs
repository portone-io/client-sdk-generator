use biome_formatter::{IndentStyle, IndentWidth, QuoteStyle};
use biome_js_formatter::{
    context::{JsFormatOptions, Semicolons, trailing_commas::TrailingCommas},
    format_node,
};
use biome_js_syntax::{JsFileSource, JsSyntaxNode};

pub fn print_node(node: &JsSyntaxNode) -> String {
    let options = JsFormatOptions::new(JsFileSource::ts())
        .with_indent_style(IndentStyle::Space)
        .with_indent_width(IndentWidth::try_from(2).unwrap())
        .with_trailing_commas(TrailingCommas::Es5)
        .with_quote_style(QuoteStyle::Single)
        .with_semicolons(Semicolons::AsNeeded);

    format_node(options, node)
        .unwrap()
        .print()
        .unwrap()
        .into_code()
}
