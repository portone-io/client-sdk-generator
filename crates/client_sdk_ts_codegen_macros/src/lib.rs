use proc_macro2::TokenStream;
use quote::{ToTokens, quote};

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
    } = syn::parse::<TsParseInput>(input).expect(
        "Failed to parse ts_parse! macro input. Expected format: \"source code\" as OutputType",
    );

    let source = match mode {
        MacroMode::TsParse => source.into_token_stream(),
        MacroMode::TsParseFunction => {
            // () => {{ ... }}
            format!("() => {{{{ {} }}}}", source.value()).into_token_stream()
        }
    };

    quote! {{
        use biome_console::{
            fmt::{Formatter, Termcolor},
            markup,
        };
        use biome_diagnostics::{DiagnosticExt, PrintDiagnostic};
        use biome_js_syntax::*;
        use biome_rowan::AstNode;
        let formatted_source = indoc::formatdoc!(#source #args);
        let parsed = biome_js_parser::parse(
            formatted_source.as_str(),
            biome_js_syntax::JsFileSource::ts(),
            biome_js_parser::JsParserOptions::default(),
        );
        let diagnostics = parsed.diagnostics();
        if !diagnostics.is_empty() {
            let mut diagnostics_buffer = termcolor::Buffer::no_color();

            let termcolor = &mut Termcolor(&mut diagnostics_buffer);
            let mut formatter = Formatter::new(termcolor);

            for diagnostic in diagnostics {
                let error = diagnostic.clone().with_file_source_code(&formatted_source);

                formatter
                    .write_markup(markup! {
                        {PrintDiagnostic::verbose(&error)}
                    })
                    .expect("failed to emit diagnostic");
            }

            let formatted_diagnostics = std::str::from_utf8(diagnostics_buffer.as_slice())
                .expect("non utf8 in error buffer");

            panic!("{}", formatted_diagnostics);
        }
        parsed.syntax()
            .descendants()
            .find_map(#output_type::cast)
            .unwrap()
    }}
    .into()
}
