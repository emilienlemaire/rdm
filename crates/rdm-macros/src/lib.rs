use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{
    parse_macro_input, DeriveInput, Expr, Field,
    Fields::{self, Unnamed},
    FieldsNamed, FieldsUnnamed, Lit, LitStr, Meta, MetaList, MetaNameValue,
    NestedMeta, Path, PathSegment, Variant,
};

/// Derives all the fields of an enum with one unnamed field with an impl of
/// the `From` trait.
///
/// # Attribute
///
/// * `no_from` - Does not derive the field even if it matches the conditions.
///
/// # Example
/// ```
/// #[derive(FromErr)]
/// enum Test {
///     ErrorNotDerived,
///     ErrorDerived(AnotherError),
///     #[no_from]
///     ErrorNotDerived2(String),
///     ErrorNotDerived3(Field1, Field2),
///     ErrorNotDerived4{
///         field: Field
///     }
/// }
/// // genrates
/// impl From<AnotherError> for Test {
///     fn from(err: AnotherError) -> Self {
///         Self::ErrorDerived(err)
///     }
/// }
/// ```
#[proc_macro_derive(FromError, attributes(no_from))]
pub fn from_error(input: TokenStream) -> TokenStream {
    let DeriveInput { ident, data, .. } = parse_macro_input!(input);

    let errs: Vec<Variant> = match data {
        syn::Data::Enum(e) => e
            .variants
            .iter()
            .cloned()
            .filter(|v| !has_no_from_attr(v) && v.fields.len() == 1)
            .collect(),
        _ => panic!("This proc macro must be used on enum only"),
    };

    if errs.is_empty() {
        return TokenStream::new();
    }

    errs.iter().fold(TokenStream::new(), |mut acc, v| {
        let Variant {
            ident: name,
            fields,
            ..
        } = v;

        if let Unnamed(FieldsUnnamed { unnamed, .. }) = fields {
            let Field { ty, .. } = &unnamed[0];
            acc.extend::<proc_macro::TokenStream>(
                quote! {
                    impl From<#ty> for #ident {
                        fn from(err: #ty) -> Self {
                            Self::#name(err)
                        }
                    }
                }
                .into(),
            );
        };

        acc
    })
}

fn has_no_from_attr(v: &Variant) -> bool {
    v.attrs.iter().any(|a| match a.parse_meta().unwrap() {
        Meta::Path(Path { segments, .. }) => {
            let PathSegment { ident, .. } = &segments[0];
            ident == "no_from"
        }
        _ => false,
    })
}

/// This deriver will create an impl for an enum to convert it to an `RcDoc<()>`
/// from the `pretty` crate.
///
/// The default behaviour works only on variant with one unnanmed field and will
/// call `.to_doc()` on this field.
///
/// # Attributes
///
/// * `doc_prefix` : This attribute must be present and expects a literal string.
///     It is used as the prefix to add before the fields of the enum are transformed.
///     It must be used on the enum itself and not the fields.
///
/// * `doc_after_prefix` : This attribute is optionnal and expects a literal string.
///     It will modify what is present after the prefix. The default is
///     `pretty::RcDoc::space()`, the string must contain an `RcDoc` method.
///     It must be used on the enum itself and not the fields.
///
/// * `doc_to_pretty` : This attribute is optionnal and enables the implementation
///     of a `to_pretty()` method.
///
/// * `doc_format` : This attribute is optionnal and must be called on each fields
///     it concerns. It enables you to use a format string for the field. The
///     first argument of the attribute must be `format_str = <a format string>`,
///     and the a list of comma seperated `_<n>` where `<n>` is a strictly positive
///     integer that will be replaced with the fields of the variant, by order
///     of declaration.
///
/// * `doc_text` : This attribute is optionnal and expects a literal string. This
///     literal string will replace the default behaviour when deriving the
///     variant it concerns.
///
/// * `doc_to_string` : This attribute is optional and can be used only on
///     variant with exactly one field (named or unnamed). It will call
///     `.to_string()` on the field of the variant.
///
/// #  Example
/// ```
/// #[derive(ToDoc)]
/// #[doc_prefix = "Error while running the `init' command:"]
/// #[doc_after_prefix = "pretty::RcDoc::line()"]
/// pub(crate) enum InitError {
///     #[doc_to_string]
///     IoError(std::io::Error),
///     #[doc_text = "There was a git error."]
///     GitError,
///     #[doc_format( format_str = "The path `{}' does not point to a directory.", _1)]
///     IsNotDirError(String),
///     LockFileError(lockfile::LockFileError),
/// }
/// // will generate
/// impl InitError {
///     pub fn to_doc(&self) -> pretty::RcDoc<()> {
///         pretty::RcDoc::text("Error while running the `init' command:")
///             .append(pretty::RcDoc::line())
///             .append(
///                 match &self {
///                     Self::IoError(ref err) => {
///                         pretty::RcDoc::text(err.to_string())
///                     }
///                     Self::GitError => {
///                         pretty::RcDoc::text("There was a git error.")
///                     }
///                     Self::IsNotDirError(_1) => {
///                         pretty::RcDoc::text(format!("The path `{}' does not \
///                         point to a directory.", _1))
///                     }
///                     Self::LockFileError(ref err) => err.to_doc(),
///                 }
///                 .nest(1),
///             )
///     }
/// }
/// ```
#[proc_macro_derive(
    ToDoc,
    attributes(
        doc_prefix,
        doc_after_prefix,
        doc_to_pretty,
        doc_format,
        doc_text,
        doc_to_string
    )
)]
pub fn to_doc(input: TokenStream) -> TokenStream {
    let DeriveInput {
        ident, data, attrs, ..
    } = parse_macro_input!(input);

    let mut prefix: Option<String> = None;
    let mut after_prefix = String::from("pretty::RcDoc::space()");
    let mut has_to_pretty = false;

    for attr in attrs.into_iter() {
        let meta_attr = attr.parse_meta().unwrap();
        if let Meta::NameValue(MetaNameValue { path, lit, .. }) = meta_attr {
            match &path.segments[0] {
                PathSegment { ident, .. } if ident == "doc_prefix" => {
                    if let Lit::Str(lit) = lit {
                        prefix = Some(lit.value())
                    } else {
                        panic!(
                            "The value of the `doc_prefix' attribute must be \
                               a string"
                        )
                    }
                }
                PathSegment { ident, .. } if ident == "doc_after_prefix" => {
                    if let Lit::Str(lit) = lit {
                        after_prefix = lit.value()
                    } else {
                        panic!(
                            "The value of the `doc_after_prefix' attribute \
                               must be a string"
                        )
                    }
                }
                _ => (),
            }
        } else if let Meta::Path(Path { segments, .. }) = meta_attr {
            let PathSegment { ident, .. } = &segments[0];
            if ident == "doc_to_pretty" {
                has_to_pretty = true;
            }
        }
    }

    if prefix.is_none() {
        panic!("The `doc_prefix' attribute must be present");
    }

    let errs: Vec<Variant> = match data {
        syn::Data::Enum(e) => e.variants.iter().cloned().collect(),
        _ => panic!("This macro can only be used on enums."),
    };

    let match_arms = errs.iter().fold(Vec::new(), |mut acc, v| {
        let Variant {
            ident: name,
            attrs,
            fields,
            ..
        } = v;

        let meta_attrs = attrs.iter().fold(Vec::new(), |mut acc, a| {
            let meta_attr = a.parse_meta().unwrap();
            acc.push(meta_attr);
            acc
        });

        acc.push(
            if let Some(str) = attr_to_text(&meta_attrs) {
                match fields {
                    Fields::Unit => quote! {
                        Self::#name => pretty::RcDoc::text(#str),
                    },
                    Fields::Unnamed(..) => quote! {
                        Self::#name(..) => pretty::RcDoc::text(#str),
                    },
                    Fields::Named(..) => quote! {
                        Self::#name{..} => pretty::RcDoc::text(#str),
                    },
                }
            } else if attr_to_string(&meta_attrs).is_some() {
                match fields {
                    Fields::Unnamed(FieldsUnnamed { unnamed, .. }) => {
                        if unnamed.len() > 1 {
                            panic!("The `doc_to_string' attribute can only be \
                                   used when there is only one field")
                        }
                        quote! {
                            Self::#name(ref err) =>
                                pretty::RcDoc::text(err.to_string()),
                        }
                    }
                    Fields::Named(FieldsNamed { named, .. }) => {
                        if named.len() > 1 {
                            panic!("The `doc_to_string' attribute can only be \
                                   used when there is only one field")
                        }

                        let field_ident = named[0].ident.as_ref().unwrap();
                        quote! {
                            Self::#name{#field_ident: ref err} =>
                                pretty::RcDoc::text(err.to_string()),
                        }
                    }
                    _ => panic!("The `doc_to_string' attribute can only be used \
                                when there is only one field")
                }
            } else if let Some(list) = attr_format(&meta_attrs) {
                let mut is_named = false;
                let fields = match fields {
                    Fields::Unit => Vec::new(),
                    Fields::Unnamed(FieldsUnnamed { unnamed, .. }) => {
                        let mut num: u32 = 1;
                        unnamed.iter().fold(Vec::new(), |mut acc, _| {
                            let id = format_ident!("_{}", num);
                            let quote = quote! {#id};
                            num += 1;
                            acc.push(quote);
                            acc
                        })
                    }
                    Fields::Named(FieldsNamed { named, .. }) => {
                        is_named = true;
                        let mut num: u32 = 1;
                        named.iter().fold(
                            Vec::new(),
                            |mut acc, Field {ref ident, ..}| {
                                let id = quote::format_ident!("_{}", num);
                                let quote = quote! {
                                    #ident: #id
                                };
                                num += 1;
                                acc.push(quote);
                                acc
                            })
                    }
                };

                let format_str = match &list.nested[0] {
                    NestedMeta::Meta(
                        Meta::NameValue(MetaNameValue { lit: Lit::Str(str), .. })
                        ) => {
                        str.value()
                    },
                    _ => unreachable!("This should have been seen before")
                };

                let nested_rest = &list.nested
                    .iter()
                    .skip(1)
                    .fold(Vec::new(), |mut acc, nest| {
                        if let NestedMeta::Meta(
                                Meta::Path(Path { segments, .. })
                        ) = nest {
                            let PathSegment { ident, .. } = &segments[0];
                            acc.push(quote!(#ident));
                            acc
                        } else {
                            panic!("All arguments after `format_str' must be \
                                   identifiers");
                        }
                    });

                if !is_named {
                    quote! {
                        Self::#name(#(#fields),*) =>
                            pretty::RcDoc::text(format!(#format_str, #(#nested_rest),*)),
                    }
                } else {
                    quote! {
                        Self::#name{#(#fields),*} =>
                            pretty::RcDoc::text(format!(#format_str, #(#nested_rest),*)),
                    }
                }
            } else if let Fields::Unnamed(FieldsUnnamed { unnamed, .. }) = fields {
                if unnamed.len() == 1 {
                    quote! {
                        Self::#name(ref err) => err.to_doc(),
                    }
                } else {
                    panic!("The default behaviour works only for a variant with \
                           one unnamed field")
                }
            } else {
                panic!("The default behaviour works only for a variant with one \
                       unnamed field")
            });
            acc
    });

    let prefix = prefix.unwrap();
    let after_prefix = syn::parse_str::<Expr>(after_prefix.as_str()).unwrap();

    if has_to_pretty {
        quote! {
            impl #ident {
                pub fn to_doc(&self) -> pretty::RcDoc<()> {
                    pretty::RcDoc::text(#prefix)
                        .append(#after_prefix)
                        .append(match &self {
                            #(#match_arms)*
                        }.nest(1))
                }

                pub fn to_pretty(&self, width: usize) -> String {
                    let mut buf = Vec::new();
                    self.to_doc().render(width, &mut buf).unwrap();
                    String::from_utf8(buf).unwrap()
                }
            }
        }
        .into()
    } else {
        quote! {
            impl #ident {
                pub fn to_doc(&self) -> pretty::RcDoc<()> {
                    pretty::RcDoc::text(#prefix)
                        .append(#after_prefix)
                        .append(match &self {
                            #(#match_arms)*
                        }.nest(1))
                }
            }
        }
        .into()
    }
}

fn attr_to_text(attrs: &[Meta]) -> Option<&LitStr> {
    let text_attr: Vec<&Meta> = attrs
        .iter()
        .filter(|&a| match a {
            Meta::NameValue(MetaNameValue { path, .. }) => {
                let PathSegment { ident, .. } = &path.segments[0];
                ident == "doc_text"
            }
            _ => false,
        })
        .collect();

    match &text_attr[..] {
        [Meta::NameValue(MetaNameValue {
            lit: Lit::Str(lit), ..
        })] => Some(lit),
        [] => None,
        _ => panic!("There must be only one `text' attribute."),
    }
}

fn attr_to_string(attrs: &[Meta]) -> Option<()> {
    let text_attr: Vec<&Meta> = attrs
        .iter()
        .filter(|&a| match a {
            Meta::Path(Path { segments, .. }) => {
                let PathSegment { ref ident, .. } = &segments[0];
                ident == "doc_to_string"
            }
            _ => false,
        })
        .collect();

    match &text_attr[..] {
        [_] => Some(()),
        [] => None,
        _ => panic!("There must be only one `text' attribute."),
    }
}

fn attr_format(attr: &[Meta]) -> Option<&MetaList> {
    let format_attr: Vec<&MetaList> = attr
        .iter()
        .filter_map(|a| match a {
            Meta::List(list) => {
                let MetaList { ref path, .. } = list;
                let PathSegment { ident, .. } = &path.segments[0];
                if ident == "doc_format" {
                    return Some(list);
                }
                None
            }
            _ => None,
        })
        .collect();

    match &format_attr[..] {
        [a] => {
            let MetaList { nested, .. } = a;
            if let NestedMeta::Meta(Meta::NameValue(MetaNameValue {
                path: Path { segments, .. },
                lit: Lit::Str(_),
                ..
            })) = &nested[0]
            {
                let PathSegment { ident, .. } = &segments[0];
                if ident == "format_str" {
                    Some(a)
                } else {
                    panic!("The `doc_format' attribute expects `format_str' as \
                           first argument and expects a literal string as value.")
                }
            } else {
                panic!(
                    "The `doc_format' attribute expects `format_str' as first \
                    argument and expects a literal string as value."
                )
            }
        }
        [] => None,
        _ => panic!("There must be only one `doc_format' attribute."),
    }
}
