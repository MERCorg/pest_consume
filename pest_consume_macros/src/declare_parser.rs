use quote::quote;
use syn::Path;
use syn::parse::Parse;
use syn::parse::ParseStream;
use syn::parse::Result;
use syn::token;

use crate::parser_methods::kw;

/// Attributes for the `declare_parser!` macro.
struct DeclareParserAttrs {
    parser: Path,
    rule_enum: Path,
}

impl Parse for DeclareParserAttrs {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut parser = None;
        // By default, use the `Rule` type in scope
        let mut rule_enum: Path = syn::parse_quote!(Rule);

        while !input.is_empty() {
            let lookahead = input.lookahead1();
            if lookahead.peek(kw::parser) {
                let _: kw::parser = input.parse()?;
                let _: token::Eq = input.parse()?;
                parser = Some(input.parse()?);
            } else if lookahead.peek(kw::rule) {
                let _: kw::rule = input.parse()?;
                let _: token::Eq = input.parse()?;
                rule_enum = input.parse()?;
            } else {
                return Err(lookahead.error());
            }

            if input.peek(token::Comma) {
                let _: token::Comma = input.parse()?;
            } else {
                break;
            }
        }

        let parser = parser.ok_or_else(|| {
            syn::Error::new(
                proc_macro2::Span::call_site(),
                "declare_parser! requires a `parser = <Type>` argument",
            )
        })?;

        Ok(DeclareParserAttrs { parser, rule_enum })
    }
}

/// Declares the `Parser` impl for a type whose rule methods are spread across multiple
/// `#[parser_methods]` impl blocks (possibly in different files), instead of the single
/// impl block the `#[parser]` macro requires.
///
/// This only supports parsers that never use `#[alias(...)]`: `AliasedRule` is just `Rule`
/// itself, `rule_alias` is the identity function, and no rule ever allows shortcutting. That
/// matches every rule method having its own one-to-one grammar rule, which is the common case.
/// A parser that needs rule aliasing must keep using a single `#[parser]` impl block instead.
pub fn declare_parser(
    input: proc_macro::TokenStream,
) -> Result<proc_macro2::TokenStream> {
    let attrs: DeclareParserAttrs = syn::parse(input)?;
    let parser = &attrs.parser;
    let rule_enum = &attrs.rule_enum;

    Ok(quote!(
        impl ::merc_pest_consume::Parser for #parser {
            type Rule = #rule_enum;
            type AliasedRule = #rule_enum;
            type Parser = #parser;
            fn rule_alias(rule: Self::Rule) -> Self::AliasedRule {
                rule
            }
            fn allows_shortcut(_rule: Self::Rule) -> bool {
                false
            }
        }
    ))
}
