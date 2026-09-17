use std::iter;

use quote::quote;
use syn::ItemImpl;
use syn::Path;
use syn::parse::Parse;
use syn::parse::ParseStream;
use syn::parse::Result;
use syn::parse_quote;
use syn::token;

use crate::parser_methods;
use crate::parser_methods::kw;

/// Attributes for the parser macro
struct MakeParserAttrs {
    parser: Path,
    rule_enum: Path,
}

impl Parse for MakeParserAttrs {
    fn parse(input: ParseStream) -> Result<Self> {
        // By default, the pest parser is the same type as the pest_consume one
        let mut parser = parse_quote!(Self);
        // By default, use the `Rule` type in scope
        let mut rule_enum = parse_quote!(Rule);

        while !input.is_empty() {
            let lookahead = input.lookahead1();
            if lookahead.peek(kw::parser) {
                let _: kw::parser = input.parse()?;
                let _: token::Eq = input.parse()?;
                parser = input.parse()?;
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

        Ok(MakeParserAttrs { parser, rule_enum })
    }
}

/// Main function for generating the parser implementation.
pub fn make_parser(
    attrs: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> Result<proc_macro2::TokenStream> {
    let attrs: MakeParserAttrs = syn::parse(attrs)?;
    let parser = &attrs.parser;
    let rule_enum = &attrs.rule_enum;
    let mut imp: ItemImpl = syn::parse(input)?;

    // Collect aliases and build rule matching logic
    let mut alias_map = parser_methods::collect_aliases(&mut imp, true)?;
    let rule_alias_branches: Vec<_> = alias_map
        .iter()
        .flat_map(|(tgt, srcs)| iter::repeat(tgt).zip(srcs))
        .map(|(tgt, src)| {
            let ident = &src.ident;
            quote!(
                #rule_enum::#ident => Self::AliasedRule::#tgt,
            )
        })
        .collect();
    let aliased_rule_variants: Vec<_> = alias_map.keys().cloned().collect();
    let shortcut_branches: Vec<_> = alias_map
        .iter()
        .flat_map(|(_tgt, srcs)| srcs)
        .map(|src| {
            let ident = &src.ident;
            let is_shortcut = src.is_shortcut;
            quote!(
                #rule_enum::#ident => #is_shortcut,
            )
        })
        .collect();

    // Process functions and apply shortcut/alias dispatch
    parser_methods::process_methods(&mut imp, rule_enum, &mut alias_map)?;

    // Generate the final implementation
    let ty = &imp.self_ty;
    let (impl_generics, _, where_clause) = imp.generics.split_for_impl();

    debug_assert!(
        !aliased_rule_variants.is_empty(),
        "Must have at least one aliased rule variant"
    );

    Ok(quote!(
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        #[allow(non_camel_case_types)]
        pub enum AliasedRule {
            #(#aliased_rule_variants,)*
        }

        impl #impl_generics ::merc_pest_consume::Parser for #ty #where_clause {
            type Rule = #rule_enum;
            type AliasedRule = AliasedRule;
            type Parser = #parser;
            fn rule_alias(rule: Self::Rule) -> Self::AliasedRule {
                match rule {
                    #(#rule_alias_branches)*
                    // TODO: return a proper error ?
                    r => panic!("Rule `{:?}` does not have a corresponding parsing method", r),
                }
            }
            fn allows_shortcut(rule: Self::Rule) -> bool {
                match rule {
                    #(#shortcut_branches)*
                    _ => false,
                }
            }
        }

        #imp
    ))
}
