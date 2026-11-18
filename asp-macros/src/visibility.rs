use proc_macro2::{Span, TokenStream, TokenTree};
use quote::{ToTokens, format_ident, quote, quote_spanned};
use syn::{
    Attribute, Error, Field, Fields, Ident, Item, ItemStruct, LitStr, Meta, Path, Result, Token, ext::IdentExt,
    parenthesized, parse::Parser, parse_quote, punctuated::Punctuated, token::Paren,
};

/// Defines the serde field attributes that cannot be combined with `#[range]`.
const CONFLICTS: [&str; 6] = [
    "with",
    "deserialize_with",
    "skip_serializing_if",
    "skip",
    "skip_serializing",
    "skip_deserializing",
];

/// Represents a member whose visibility is limited to a set of API version ranges.
struct Member {
    ident: Ident,
    ranges: Vec<TokenStream>,
}

pub fn expand(args: TokenStream, item: TokenStream) -> Result<TokenStream> {
    let krate = parse_args(args)?;
    let mut item = match syn::parse2(item)? {
        Item::Struct(item) => item,
        item => {
            return Err(Error::new_spanned(
                item,
                "#[visible_in_version] can only be applied to a struct",
            ));
        }
    };

    if !has_serde_derive(&item.attrs) {
        return Err(Error::new(
            Span::call_site(),
            "#[visible_in_version] must be placed before #[derive(Serialize, Deserialize)]",
        ));
    }

    let module = format_ident!("__visible_in_version_{}", item.ident.unraw());
    let members = rewrite_fields(&mut item, &module, &krate)?;

    if members.is_empty() {
        return Ok(quote!(#item));
    }

    let (schema, helpers) = match take_to_schema(&mut item.attrs) {
        Some(derive) => expand_schema(&mut item, &derive, &module, &members, &krate),
        None => Default::default(),
    };
    let members = members.iter().map(|Member { ident, ranges }| {
        quote! {
            pub mod #ident {
                pub fn contains(version: &#krate::ApiVersion<'static>) -> bool {
                    #(::core::ops::RangeBounds::contains(&(#ranges), version))||*
                }

                pub fn is_hidden<T: ?::core::marker::Sized>(_: &T) -> bool {
                    !#krate::visibility::is_visible(contains)
                }

                pub fn deserialize<'de, D, T>(deserializer: D) -> ::core::result::Result<T, D::Error>
                where
                    D: #krate::__serde::Deserializer<'de>,
                    T: #krate::__serde::Deserialize<'de>,
                {
                    #krate::visibility::deserialize(deserializer, contains)
                }
            }
        }
    });

    Ok(quote! {
        #item

        #schema

        #[doc(hidden)]
        #[allow(non_snake_case)]
        mod #module {
            #(#members)*

            #helpers
        }
    })
}

/// Removes the `ToSchema` derive from the attributes, if any, and returns its path.
fn take_to_schema(attrs: &mut Vec<Attribute>) -> Option<Path> {
    for (index, attr) in attrs.iter().enumerate() {
        if !attr.path().is_ident("derive") {
            continue;
        }

        let Ok(paths) = attr.parse_args_with(Punctuated::<Path, Token![,]>::parse_terminated) else {
            continue;
        };
        let is_to_schema = |path: &Path| path.segments.last().is_some_and(|segment| segment.ident == "ToSchema");
        let Some(derive) = paths.iter().find(|path| is_to_schema(path)).cloned() else {
            continue;
        };
        let remaining: Vec<_> = paths.into_iter().filter(|path| !is_to_schema(path)).collect();

        if remaining.is_empty() {
            attrs.remove(index);
        } else {
            attrs[index] = parse_quote!(#[derive(#(#remaining),*)]);
        }

        return Some(derive);
    }

    None
}

/// Determines whether an attribute is understood by the `ToSchema` derive.
fn is_schema_attr(attr: &Attribute) -> bool {
    let path = attr.path();
    path.is_ident("schema")
        || path.is_ident("serde")
        || (path.is_ident("doc") && matches!(attr.meta, Meta::NameValue(_)))
}

/// Determines whether the attributes define the name of the schema with `#[schema(as = ..)]`.
fn has_schema_name(attrs: &[Attribute]) -> bool {
    attrs
        .iter()
        .filter(|attr| attr.path().is_ident("schema"))
        .any(|attr| match &attr.meta {
            Meta::List(list) => list
                .tokens
                .clone()
                .into_iter()
                .any(|token| matches!(token, TokenTree::Ident(ident) if ident == "as")),
            _ => false,
        })
}

/// Expands the OpenAPI schema of a struct so that it only has the members visible in the current API version.
///
/// # Remarks
///
/// The schema is derived for hidden copies of the struct: one with every member and one for each member whose
/// visibility is limited, which omits that member. The property names of a member are the difference between the
/// two schemas, which accounts for every rename that `ToSchema` applies. The struct implements `ToSchema` by removing
/// the properties of the members that are not visible in the current API version, if any.
fn expand_schema(
    item: &mut ItemStruct,
    derive: &Path,
    module: &Ident,
    members: &[Member],
    krate: &Path,
) -> (TokenStream, TokenStream) {
    let ident = item.ident.clone();
    let name = ident.unraw();
    let full = format_ident!("__{}Schema", name);
    let shadows: Vec<_> = members
        .iter()
        .map(|member| format_ident!("__{}Schema_{}", name, member.ident.unraw()))
        .collect();
    let copy = |shadow: &Ident, omitted: Option<&Ident>| {
        let mut copy = item.clone();

        copy.ident = shadow.clone();
        copy.attrs.retain(is_schema_attr);

        if !has_schema_name(&copy.attrs) {
            copy.attrs.push(parse_quote!(#[schema(as = #ident)]));
        }

        copy.attrs.insert(0, parse_quote!(#[derive(#derive)]));
        copy.attrs.push(parse_quote!(#[doc(hidden)]));
        copy.attrs.push(parse_quote!(#[allow(dead_code, non_camel_case_types)]));

        for field in &mut copy.fields {
            field.attrs.retain(is_schema_attr);

            if field.ident.as_ref() == omitted {
                field.attrs.push(parse_quote!(#[schema(ignore)]));
            }
        }

        copy
    };
    let copies: Vec<_> = std::iter::once(copy(&full, None))
        .chain(
            members
                .iter()
                .zip(&shadows)
                .map(|(member, shadow)| copy(shadow, Some(&member.ident))),
        )
        .collect();

    item.attrs.retain(|attr| !attr.path().is_ident("schema"));

    for field in &mut item.fields {
        field.attrs.retain(|attr| !attr.path().is_ident("schema"));
    }

    let (impl_generics, ty_generics, _) = item.generics.split_for_impl();
    let bounds = |bound: TokenStream, extra: Option<TokenStream>| {
        let mut generics = item.generics.clone();
        let clause = generics.make_where_clause();

        for ty in std::iter::once(&full).chain(&shadows) {
            clause.predicates.push(parse_quote!(#ty #ty_generics: #bound));
        }

        if let Some(extra) = extra {
            clause.predicates.push(parse_quote!(#extra));
        }

        clause.to_token_stream()
    };
    let compose_where = bounds(quote!(utoipa::__dev::ComposeSchema), None);
    let to_schema_where = bounds(
        quote!(utoipa::ToSchema),
        Some(quote!(Self: utoipa::__dev::ComposeSchema)),
    );
    let contains: Vec<_> = members.iter().map(|member| &member.ident).collect();
    let schema = quote! {
        #(#copies)*

        impl #impl_generics utoipa::__dev::ComposeSchema for #ident #ty_generics #compose_where {
            fn compose(
                generics: ::std::vec::Vec<utoipa::openapi::RefOr<utoipa::openapi::schema::Schema>>,
            ) -> utoipa::openapi::RefOr<utoipa::openapi::schema::Schema> {
                let mut schema = <#full #ty_generics as utoipa::__dev::ComposeSchema>::compose(generics.clone());

                #(
                    if !#krate::visibility::is_visible(#module::#contains::contains) {
                        let visible = <#shadows #ty_generics as utoipa::__dev::ComposeSchema>::compose(generics.clone());
                        #module::retain_properties(&mut schema, &visible);
                    }
                )*

                schema
            }
        }

        impl #impl_generics utoipa::ToSchema for #ident #ty_generics #to_schema_where {
            fn name() -> ::std::borrow::Cow<'static, str> {
                <#full #ty_generics as utoipa::ToSchema>::name()
            }

            fn schemas(
                schemas: &mut ::std::vec::Vec<(
                    ::std::string::String,
                    utoipa::openapi::RefOr<utoipa::openapi::schema::Schema>,
                )>,
            ) {
                let start = schemas.len();

                <#full #ty_generics as utoipa::ToSchema>::schemas(schemas);

                #(
                    if !#krate::visibility::is_visible(#module::#contains::contains) {
                        let mut visible = ::std::vec::Vec::new();
                        <#shadows #ty_generics as utoipa::ToSchema>::schemas(&mut visible);
                        #module::retain_schemas(schemas, start, &visible);
                    }
                )*
            }
        }
    };
    let helpers = quote! {
        /// Removes the properties of a schema that are not defined by the schema that is visible.
        pub fn retain_properties(
            schema: &mut utoipa::openapi::RefOr<utoipa::openapi::schema::Schema>,
            visible: &utoipa::openapi::RefOr<utoipa::openapi::schema::Schema>,
        ) {
            use utoipa::openapi::{RefOr, schema::Schema};

            if let (RefOr::T(Schema::Object(schema)), RefOr::T(Schema::Object(visible))) = (schema, visible) {
                schema.properties.retain(|name, _| visible.properties.contains_key(name));
                schema.required.retain(|name| visible.properties.contains_key(name));
            }
        }

        /// Removes the schemas added after the specified index that are not referenced by the schema that is visible.
        pub fn retain_schemas(
            schemas: &mut ::std::vec::Vec<(::std::string::String, utoipa::openapi::RefOr<utoipa::openapi::schema::Schema>)>,
            start: usize,
            visible: &[(::std::string::String, utoipa::openapi::RefOr<utoipa::openapi::schema::Schema>)],
        ) {
            let mut index = 0;

            schemas.retain(|(name, _)| {
                let retain = index < start || visible.iter().any(|(other, _)| other == name);
                index += 1;
                retain
            });
        }
    };

    (schema, helpers)
}

fn parse_args(args: TokenStream) -> Result<Path> {
    let mut krate = parse_quote!(::api_versioning);
    let parser = syn::meta::parser(|meta| {
        if meta.path.is_ident("crate") {
            krate = meta.value()?.parse::<LitStr>()?.parse()?;
            Ok(())
        } else {
            Err(meta.error("unsupported #[visible_in_version] argument"))
        }
    });

    parser.parse2(args)?;
    Ok(krate)
}

/// Determines whether the attributes derive serde traits. A derive that runs before `#[visible_in_version]` never sees
/// the serde attributes it adds, so the derive must still be present. `cfg_attr` cannot be evaluated, so it is trusted.
fn has_serde_derive(attrs: &[Attribute]) -> bool {
    attrs.iter().any(|attr| {
        if attr.path().is_ident("cfg_attr") {
            return true;
        }

        attr.path().is_ident("derive")
            && attr
                .parse_args_with(Punctuated::<Path, Token![,]>::parse_terminated)
                .is_ok_and(|paths| {
                    paths.iter().any(|path| {
                        path.segments
                            .last()
                            .is_some_and(|segment| segment.ident == "Serialize" || segment.ident == "Deserialize")
                    })
                })
    })
}

fn rewrite_fields(item: &mut ItemStruct, module: &Ident, krate: &Path) -> Result<Vec<Member>> {
    let Fields::Named(fields) = &mut item.fields else {
        return Err(Error::new_spanned(
            &item.fields,
            "#[visible_in_version] can only be applied to a struct with named fields",
        ));
    };
    let mut members = Vec::new();

    for field in &mut fields.named {
        let mut ranges = None;
        let mut remaining = Vec::with_capacity(field.attrs.len());

        for attr in field.attrs.drain(..) {
            if !attr.path().is_ident("range") {
                remaining.push(attr);
            } else if ranges.is_some() {
                return Err(Error::new_spanned(attr, "a member can only have one #[range]"));
            } else {
                ranges = Some(parse_ranges(&attr, krate)?);
            }
        }

        field.attrs = remaining;

        if let Some(ranges) = ranges {
            let ident = field.ident.clone().expect("named fields have an identifier");
            add_serde_attribute(field, module, &ident)?;
            members.push(Member { ident, ranges });
        }
    }

    Ok(members)
}

fn add_serde_attribute(field: &mut Field, module: &Ident, ident: &Ident) -> Result<()> {
    let mut default = true;

    for attr in field.attrs.iter().filter(|attr| attr.path().is_ident("serde")) {
        attr.parse_nested_meta(|meta| {
            if meta.input.peek(Token![=]) {
                meta.value()?.parse::<syn::Expr>()?;
            } else if meta.input.peek(Paren) {
                let content;
                parenthesized!(content in meta.input);
                content.parse::<TokenStream>()?;
            }

            if meta.path.is_ident("default") {
                default = false;
            } else if let Some(name) = CONFLICTS.iter().find(|name| meta.path.is_ident(name)) {
                return Err(Error::new_spanned(
                    &meta.path,
                    format!("#[range] cannot be combined with #[serde({name})]"),
                ));
            }

            Ok(())
        })?;
    }

    let default = default.then(|| quote!(default,));
    let is_hidden = LitStr::new(&format!("{module}::{ident}::is_hidden"), ident.span());
    let deserialize = LitStr::new(&format!("{module}::{ident}::deserialize"), ident.span());

    field.attrs.push(parse_quote! {
        #[serde(#default skip_serializing_if = #is_hidden, deserialize_with = #deserialize)]
    });

    Ok(())
}

fn parse_ranges(attr: &Attribute, krate: &Path) -> Result<Vec<TokenStream>> {
    let Meta::List(list) = &attr.meta else {
        return Err(Error::new_spanned(
            attr,
            "expected #[range(..)] with at least one range",
        ));
    };
    let mut ranges = Vec::new();
    let mut chunk = Vec::new();
    let mut tokens = list.tokens.clone().into_iter().peekable();

    while let Some(token) = tokens.next() {
        let separator = matches!(&token, TokenTree::Punct(punct) if punct.as_char() == ',');

        if !separator {
            chunk.push(token);
        }

        if separator || tokens.peek().is_none() {
            if chunk.is_empty() {
                return Err(Error::new_spanned(&list.tokens, "expected a range of API versions"));
            }

            ranges.push(parse_range(&chunk, krate)?);
            chunk.clear();
        }
    }

    if ranges.is_empty() {
        return Err(Error::new_spanned(
            attr,
            "expected #[range(..)] with at least one range",
        ));
    }

    Ok(ranges)
}

fn parse_range(tokens: &[TokenTree], krate: &Path) -> Result<TokenStream> {
    let span = tokens[0].span();
    let text: String = tokens.iter().map(ToString::to_string).collect();
    let error = |message: &str| {
        let tokens: TokenStream = tokens.iter().cloned().collect();
        Error::new_spanned(tokens, message)
    };
    let Some((start, end)) = text.split_once("..") else {
        return Err(error(
            "expected a range of API versions, such as `2.0..` or `2.0..=2.0`, rather than a single API version",
        ));
    };
    let (inclusive, end) = match end.strip_prefix('=') {
        Some(end) => (true, end),
        None => (false, end),
    };

    if end.contains("..") {
        return Err(error("expected a single range of API versions"));
    }

    if start.is_empty() && end.is_empty() {
        return Err(error("a range of API versions must have a start, an end, or both"));
    }

    if inclusive && end.is_empty() {
        return Err(error("an inclusive range of API versions must have an end"));
    }

    let version = |text: &str| {
        let text = LitStr::new(text, span);
        quote_spanned! {span=>
            {
                use #krate::ApiVersion as Version;
                const VERSION: Version<'static> = Version::parse_literal(#text);
                VERSION
            }
        }
    };
    let start = (!start.is_empty()).then(|| version(start));
    let end = (!end.is_empty()).then(|| version(end));

    Ok(if inclusive {
        quote!(#start..=#end)
    } else {
        quote!(#start..#end)
    })
}
