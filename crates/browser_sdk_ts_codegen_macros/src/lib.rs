use proc_macro2::TokenStream;
use quote::{quote, ToTokens};

enum MacroMode {
    TsParse,
    TsParseFunction,
}

struct TsParseInput {
    source: syn::LitStr,
    _as: syn::Token![as],
    output_type: syn::Type,
    args: TokenStream,
}

impl syn::parse::Parse for TsParseInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(TsParseInput {
            source: input.parse()?,
            _as: input.parse()?,
            output_type: input.parse()?,
            args: input.parse()?,
        })
    }
}

#[proc_macro]
pub fn ts_parse(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    expand(input, MacroMode::TsParse)
}

/// 함수 내부에 있지 않은 상태에서 `JsReturnStatement`를 파싱하면 에러가 발생하기 때문에 함수 내부에 있는 상태로 파싱하도록 하는 매크로
#[proc_macro]
pub fn ts_parse_function(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    expand(input, MacroMode::TsParseFunction)
}

fn expand(input: proc_macro::TokenStream, mode: MacroMode) -> proc_macro::TokenStream {
    let TsParseInput {
        source,
        _as: _,
        output_type,
        args,
    } = syn::parse::<TsParseInput>(input).unwrap();

    let source = match mode {
        MacroMode::TsParse => source.into_token_stream(),
        MacroMode::TsParseFunction => {
            // () => {{ ... }}
            format!("() => {{{{ {} }}}}", source.value()).into_token_stream()
        }
    };

    quote! {{
        use biome_js_syntax::*;
        use biome_rowan::AstNode;
        let node = biome_js_parser::parse(
            indoc::formatdoc!(#source #args).as_str(),
            biome_js_syntax::JsFileSource::ts(),
            biome_js_parser::JsParserOptions::default(),
        );
        if node.has_errors() {
            let error = node
                .into_diagnostics()
                .into_iter()
                .map(|d| biome_diagnostics::print_diagnostic_to_string(&d.into()))
                .collect::<Vec<_>>()
                .join("\n");
            panic!("Error parsing ts: {}", error);
        }
        node.syntax()
            .descendants()
            .find_map(#output_type::cast)
            .unwrap()
    }}
    .into()
}
