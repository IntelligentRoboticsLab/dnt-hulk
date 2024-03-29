use std::collections::HashSet;

use proc_macro2::TokenStream;
use proc_macro_error::{abort, proc_macro_error};
use quote::{quote, ToTokens};
use syn::{
    parse_macro_input, punctuated::Punctuated, Data, DataStruct, DeriveInput, Generics, Ident, Lit,
    Meta, MetaNameValue, NestedMeta, Token, Type, WherePredicate,
};

#[proc_macro_derive(SerializeHierarchy, attributes(serialize_hierarchy))]
#[proc_macro_error]
pub fn serialize_hierarchy(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    process_input(input).into()
}

fn process_input(mut input: DeriveInput) -> TokenStream {
    let fields = match &input.data {
        Data::Struct(data) => read_fields(data),
        Data::Enum(..) => Vec::new(),
        Data::Union(data) => {
            abort!(
                data.union_token,
                "`SerializeHierarchy` can only be derived for `struct` or `enum`",
            )
        }
    };
    let type_attributes = parse_attributes(&input.attrs);
    let contains_as_jpeg = type_attributes.contains(&TypeAttribute::AsJpeg);

    extend_where_clause_from_attributes(&mut input.generics, type_attributes);

    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let serializable_fields: Vec<_> = fields
        .iter()
        .filter(|field| !field.attributes.contains(&FieldAttribute::Skip))
        .collect();
    let path_serializations = generate_path_serializations(&serializable_fields);
    let serde_serializations = generate_serde_serializations(&serializable_fields);
    let path_deserializations = generate_path_deserializations(&serializable_fields);
    let serde_deserializations = generate_serde_deserializations(&serializable_fields);
    let path_exists_getters = generate_path_exists_getters(&serializable_fields);
    let field_exists_getters = generate_field_exists_getters(&serializable_fields);
    let field_chains = generate_field_chains(&serializable_fields);
    let path_field_chains = generate_path_field_chains(&serializable_fields);
    let (jpeg_serialization, jpeg_exists_getter, jpeg_field_chain) = if contains_as_jpeg {
        (
            quote! {
                "jpeg" => self
                    .encode_as_jpeg(Self::DEFAULT_QUALITY)
                    .map_err(|error| serialize_hierarchy::Error::SerializationFailed(serde::ser::Error::custom(error)))?
                    .serialize(serializer)
                    .map_err(serialize_hierarchy::Error::SerializationFailed),
            },
            quote! {
                "jpeg" => true,
            },
            quote! {
                .chain(std::iter::once("jpeg".to_string()))
            },
        )
    } else {
        Default::default()
    };

    let implementation = quote! {
        impl #impl_generics serialize_hierarchy::SerializeHierarchy for #name #ty_generics #where_clause {
            fn serialize_path<S>(
                &self,
                path: &str,
                serializer: S,
            ) -> Result<S::Ok, serialize_hierarchy::Error<S::Error>>
            where
                S: serde::Serializer,
            {
                let split = path.split_once('.');
                match split {
                    Some((name, suffix)) => match name {
                        #(#path_serializations,)*
                        segment => Err(serialize_hierarchy::Error::UnexpectedPathSegment {
                            segment: segment.to_string(),
                        }),
                    },
                    None => {
                        match path {
                            #(#serde_serializations,)*
                            #jpeg_serialization
                            segment => Err(serialize_hierarchy::Error::UnexpectedPathSegment {
                                segment: segment.to_string(),
                            }),
                        }
                    }
                }
            }

            fn deserialize_path<'de, D>(
                &mut self,
                path: &str,
                deserializer: D,
            ) -> Result<(), serialize_hierarchy::Error<D::Error>>
            where
                D: serde::Deserializer<'de>,
            {
                let split = path.split_once('.');
                match split {
                    Some((name, suffix)) => match name {
                        #(#path_deserializations,)*
                        name => Err(serialize_hierarchy::Error::UnexpectedPathSegment {
                            segment: name.to_string(),
                        }),
                    },
                    None => match path {
                        #(#serde_deserializations,)*
                        name => Err(serialize_hierarchy::Error::UnexpectedPathSegment {
                            segment: name.to_string(),
                        }),
                    },
                }
            }

            fn exists(path: &str) -> bool {
                let split = path.split_once('.');
                match split {
                    Some((name, suffix)) => match name {
                        #(#path_exists_getters,)*
                        _ => false,
                    },
                    None => match path {
                        #(#field_exists_getters,)*
                        #jpeg_exists_getter
                        _ => false,
                    },
                }
            }

            fn get_fields() -> std::collections::BTreeSet<String> {
                std::iter::empty::<std::string::String>()
                    #(#field_chains)*
                    #(#path_field_chains)*
                    #jpeg_field_chain
                    .collect()
            }
        }
    };
    implementation
}

fn extend_where_clause_from_attributes(
    generics: &mut Generics,
    type_attributes: HashSet<TypeAttribute>,
) {
    generics.make_where_clause().predicates.extend({
        type_attributes
            .iter()
            .filter_map(|attribute| match attribute {
                TypeAttribute::Bounds { predicates } => Some(predicates),
                _ => None,
            })
            .flatten()
            .cloned()
    });
}

fn generate_path_serializations(fields: &[&Field]) -> Vec<TokenStream> {
    fields
        .iter()
        .filter(|field| !field.attributes.contains(&FieldAttribute::Leaf))
        .map(|field| {
            let identifier = &field.identifier;
            let pattern = identifier.to_string();
            quote! {
                #pattern => self.#identifier.serialize_path(suffix, serializer)
            }
        })
        .collect()
}

fn generate_serde_serializations(fields: &[&Field]) -> Vec<TokenStream> {
    fields.iter().map(|field| {
        let identifier = &field.identifier;
        let pattern = identifier.to_string();
        quote! {
            #pattern => serde::Serialize::serialize(&self.#identifier, serializer).map_err(serialize_hierarchy::Error::SerializationFailed)
        }
    }).collect()
}

fn generate_path_deserializations(fields: &[&Field]) -> Vec<TokenStream> {
    fields
        .iter()
        .filter(|field| !field.attributes.contains(&FieldAttribute::Leaf))
        .map(|field| {
            let identifier = &field.identifier;
            let pattern = identifier.to_string();
            quote! {
                #pattern => self.#identifier.deserialize_path(suffix, deserializer)
            }
        })
        .collect()
}

fn generate_serde_deserializations(fields: &[&Field]) -> Vec<TokenStream> {
    fields.iter().map(|field| {
        let identifier = &field.identifier;
        let pattern = identifier.to_string();
        let ty = &field.ty;
        quote! {
            #pattern => {
                self.#identifier = <#ty as serde::Deserialize>::deserialize(deserializer).map_err(serialize_hierarchy::Error::DeserializationFailed)?;
                Ok(())
            }

        }
    }).collect()
}

fn generate_path_exists_getters(fields: &[&Field]) -> Vec<TokenStream> {
    fields
        .iter()
        .filter(|field| !field.attributes.contains(&FieldAttribute::Leaf))
        .map(|field| {
            let pattern = field.identifier.to_string();
            let ty = &field.ty;
            quote! {
                #pattern => <#ty as serialize_hierarchy::SerializeHierarchy>::exists(suffix)
            }
        })
        .collect()
}

fn generate_field_exists_getters(fields: &[&Field]) -> Vec<TokenStream> {
    fields
        .iter()
        .map(|field| {
            let pattern = field.identifier.to_string();
            quote! {
                #pattern => true
            }
        })
        .collect()
}

fn generate_field_chains(fields: &[&Field]) -> Vec<TokenStream> {
    fields
        .iter()
        .map(|field| {
            let name_string = field.identifier.to_string();
            quote! {
                .chain(std::iter::once(#name_string.to_string()))
            }
        })
        .collect()
}

fn generate_path_field_chains(fields: &[&Field]) -> Vec<TokenStream> {
    fields
        .iter()
        .filter(|field| !field.attributes.contains(&FieldAttribute::Leaf))
        .map(|field| {
            let identifier = &field.identifier;
            let pattern = format!("{identifier}.{{}}");
            let ty = &field.ty;
            quote! {
                .chain(
                    <#ty as serialize_hierarchy::SerializeHierarchy>::get_fields()
                        .into_iter()
                        .map(|name| format!(#pattern, name))
                )
            }
        })
        .collect()
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
enum TypeAttribute {
    AsJpeg,
    Bounds { predicates: Vec<WherePredicate> },
}

fn parse_attributes(attrs: &[syn::Attribute]) -> HashSet<TypeAttribute> {
    attrs
        .iter()
        .flat_map(parse_meta_items)
        .map(|meta| match meta {
            NestedMeta::Meta(Meta::Path(word)) if word.is_ident("as_jpeg") => TypeAttribute::AsJpeg,
            NestedMeta::Meta(Meta::NameValue(MetaNameValue {
                path, lit: literal, ..
            })) if path.is_ident("bound") => {
                let string = match literal {
                    Lit::Str(literal) => literal,
                    _ => abort!(
                        literal,
                        "expected bound attribute to be a string: `bound = \"...\"`"
                    ),
                };
                let predicates = match string
                    .parse_with(Punctuated::<WherePredicate, Token![,]>::parse_terminated)
                {
                    Ok(predicates) => Vec::from_iter(predicates),
                    Err(error) => {
                        abort!(error.span(), error.to_string())
                    }
                };
                TypeAttribute::Bounds { predicates }
            }
            NestedMeta::Meta(meta_item) => {
                let path = meta_item
                    .path()
                    .into_token_stream()
                    .to_string()
                    .replace(' ', "");
                abort!(meta_item.path(), "unknown attribute `{}`", path)
            }
            NestedMeta::Lit(lit) => {
                abort!(lit, "unexpected literal in attribute")
            }
        })
        .collect()
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
enum FieldAttribute {
    Skip,
    Leaf,
}

#[derive(Debug)]
struct Field {
    attributes: HashSet<FieldAttribute>,
    identifier: Ident,
    ty: Type,
}

fn parse_meta_items(attribute: &syn::Attribute) -> Vec<NestedMeta> {
    if !attribute.path.is_ident("serialize_hierarchy") {
        return Vec::new();
    }
    match attribute.parse_meta() {
        Ok(Meta::List(meta)) => meta.nested.into_iter().collect(),
        Ok(other) => abort!(other, "expected `#[serialize_hierarchy(...)]`",),
        Err(error) => abort!(error.span(), error.to_string()),
    }
}

fn read_fields(input: &DataStruct) -> Vec<Field> {
    input
        .fields
        .iter()
        .map(|field| {
            let attributes = field
                .attrs
                .iter()
                .flat_map(parse_meta_items)
                .map(|meta| match meta {
                    NestedMeta::Meta(Meta::Path(word)) if word.is_ident("skip") => {
                        FieldAttribute::Skip
                    }
                    NestedMeta::Meta(Meta::Path(word)) if word.is_ident("leaf") => {
                        FieldAttribute::Leaf
                    }
                    NestedMeta::Meta(meta_item) => {
                        let path = meta_item
                            .path()
                            .into_token_stream()
                            .to_string()
                            .replace(' ', "");
                        abort!(meta_item.path(), "unknown attribute `{}`", path)
                    }

                    NestedMeta::Lit(lit) => {
                        abort!(lit, "unexpected literal in attribute")
                    }
                })
                .collect();
            let identifier = field
                .ident
                .clone()
                .unwrap_or_else(|| abort!(field, "field has to be named"));
            let ty = field.ty.clone();
            Field {
                attributes,
                identifier,
                ty,
            }
        })
        .collect()
}
