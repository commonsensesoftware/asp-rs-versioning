use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{
    Error, Path, PathArguments, Result, Token,
    parse::{Parse, ParseStream},
};

/// Represents the input of the macro, which is the path to the crate that defines `__private::describe` followed by
/// the path to a handler annotated with `#[utoipa::path]`.
struct Input {
    krate: TokenStream,
    handler: Path,
}

impl Parse for Input {
    fn parse(input: ParseStream) -> Result<Self> {
        let krate = input.step(|cursor| {
            let mut tokens = TokenStream::new();
            let mut rest = *cursor;

            while let Some((token, next)) = rest.token_tree() {
                if matches!(&token, proc_macro2::TokenTree::Punct(punct) if punct.as_char() == ';') {
                    return Ok((tokens, rest));
                }

                tokens.extend([token]);
                rest = next;
            }

            Err(cursor.error("expected `;`"))
        })?;

        input.parse::<Token![;]>()?;

        let handler = input.parse::<Path>().map_err(|error| {
            Error::new(
                error.span(),
                "expected the path to a handler function annotated with #[utoipa::path]",
            )
        })?;

        input.parse::<Option<Token![,]>>()?;

        if !input.is_empty() {
            return Err(input.error("expected the path to a single handler function"));
        }

        Ok(Self { krate, handler })
    }
}

pub fn expand(input: TokenStream) -> Result<TokenStream> {
    let Input { krate, handler } = syn::parse2(input)?;
    let mut operation = handler.clone();
    let last = operation.segments.last_mut().expect("a path has at least one segment");

    if !matches!(last.arguments, PathArguments::None) {
        return Err(Error::new_spanned(
            &last.arguments,
            "a handler annotated with #[utoipa::path] cannot have generic arguments",
        ));
    }

    // #[utoipa::path] generates a type named `__path_<fn>` next to the function it annotates
    last.ident = format_ident!("__path_{}", last.ident, span = last.ident.span());

    Ok(quote! {
        #krate::__private::describe::<#operation, _>(#handler)
    })
}
