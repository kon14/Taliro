extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{
    ExprPath, Ident, Path, Token,
    parse::{Parse, ParseStream},
    parse_macro_input,
    punctuated::Punctuated,
};

/// Represents a single logging function definition.
struct LogFnItem {
    fn_name: Ident,
    level: Ident,
    target: ExprPath,
}

impl Parse for LogFnItem {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // Parses content within parentheses `()`
        let content;
        syn::parenthesized!(content in input);

        // Inside the parentheses, we parse the three parts separated by commas.
        let fn_name: Ident = content.parse()?;
        content.parse::<Token![,]>()?;
        let level: Ident = content.parse()?;
        content.parse::<Token![,]>()?;
        let target: ExprPath = content.parse()?;

        Ok(LogFnItem {
            fn_name,
            level,
            target,
        })
    }
}

/// Represents the entire input to the procedural macro.
/// e.g., `log, (item1), (item2), (item3)`
struct MacroInput {
    log_path: Path,
    items: Punctuated<LogFnItem, Token![,]>,
}

impl Parse for MacroInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // Parse the path to the log crate.
        let log_path = input.parse()?;
        // Parse the comma that separates the path from the list.
        input.parse::<Token![,]>()?;

        let items = Punctuated::parse_terminated(input)?;

        Ok(Self { log_path, items })
    }
}

/// Defines multiple logging macros with specified names, levels, and targets.<br />
/// Example usage:<br />
/// `define_log_macros![path::to::base::log, (log_info, info, "target"), (log_warn, warn, "target")];`.
#[proc_macro]
pub fn define_log_macros(input: TokenStream) -> TokenStream {
    let MacroInput { log_path, items } = parse_macro_input!(input as MacroInput);

    let expanded = items.iter().map(
        |LogFnItem {
             fn_name,
             level,
             target,
         }| {
            quote! {
                #[macro_export]
                macro_rules! #fn_name {
                    ($($arg:tt)*) => {
                        #log_path::#level!(target: #target, $($arg)*);
                    };
                }
            }
        },
    );

    TokenStream::from(quote! {
        #(#expanded)*
    })
}
