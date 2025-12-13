use super::attr;
use crate::internals::Ctxt;

pub struct Metadata<'a> {
    pub ident: syn::Ident,
    pub attrs: attr::Metadata,
    pub fields: Vec<Field<'a>>,
    pub generics: &'a syn::Generics,
    #[allow(dead_code)]
    pub original: &'a syn::DeriveInput,
}

pub struct Field<'a> {
    pub ident: syn::Ident,
    pub attrs: attr::FieldMetadata,
    pub ty: &'a syn::Type,
    #[allow(dead_code)]
    pub original: &'a syn::Field,
}

impl<'a> Metadata<'a> {
    pub fn from_ast(cx: &Ctxt, ast: &'a syn::DeriveInput) -> Option<Metadata<'a>> {
        let attrs = attr::Metadata::from_ast(cx, ast);
        let fields = match ast.data {
            syn::Data::Struct(syn::DataStruct {
                fields: syn::Fields::Named(ref fields),
                ..
            }) => fields.named.iter().map(|field| Field {
                ident: field.ident.to_owned().unwrap(), // NonNone
                attrs: attr::FieldMetadata::from_ast(cx, field),
                ty: &field.ty,
                original: field,
            }),
            _ => {
                cx.error_spanned_by(
                    ast,
                    "`NativeBridge` can only be derived for structs with named fields.",
                );
                return None;
            }
        }
        .collect::<Vec<_>>();

        Some(Self {
            ident: ast.ident.to_owned(),
            attrs,
            fields,
            generics: &ast.generics,
            original: ast,
        })
    }
}
