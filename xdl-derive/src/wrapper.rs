use crate::internals::{Ctxt, ast, attr};
use proc_macro2::TokenStream;
use quote::{ToTokens, format_ident, quote};
use std::ffi::CString;
use syn::DeriveInput;

pub fn expand_derive(ast: &DeriveInput) -> syn::Result<TokenStream> {
    let ctxt = Ctxt::new();
    let Some(metadata) = ast::Metadata::from_ast(&ctxt, ast) else {
        return Err(ctxt.check().unwrap_err());
    };
    let ident = &metadata.ident;
    let (impl_generics, ty_generics, where_clause) = metadata.generics.split_for_impl();
    let symbols_load_clause = generate_symbols_load_clause(&ctxt, &metadata);
    let symbols_wrapper = generate_symbols_wrapper(&ctxt, &metadata);
    let tokens = quote! {
        impl #impl_generics ::android_xdl::wrapper::Symbols for #ident #ty_generics #where_clause {
            unsafe fn load_from(lib: &::android_xdl::raw::Library) -> ::std::result::Result<Self, ::android_xdl::Error> {
                Ok(Self { #symbols_load_clause })
            }
        }
        #[allow(dead_code)]
        impl #impl_generics #ident #ty_generics #where_clause {
            #symbols_wrapper
        }
    };
    ctxt.check()?;
    Ok(tokens)
}

fn generate_symbols_load_clause(cx: &Ctxt, meta: &ast::Metadata) -> TokenStream {
    let mut tokens = TokenStream::new();
    for field in &meta.fields {
        match skip_type_group(field.ty) {
            syn::Type::BareFn(_) | syn::Type::Reference(_) | syn::Type::Ptr(_) => {
                let ident = &field.ident;
                let expr = generate_symbols_load_expr(field, meta);
                tokens.extend(quote!(#ident: #expr?,))
            }
            syn::Type::Path(ty) if get_option_inner_type(ty).is_some() => {
                let ident = &field.ident;
                let expr = generate_symbols_load_expr(field, meta);
                tokens.extend(quote!(#ident: #expr.ok(),))
            }
            _ => {
                cx.error_spanned_by(
                    field.ty,
                    format!(
                        "unsupported type: `{}`, expected one of function, reference, pointer",
                        field.ty.into_token_stream()
                    ),
                );
            }
        }
    }
    tokens
}

fn generate_symbols_load_expr(field: &ast::Field, meta: &ast::Metadata) -> TokenStream {
    let symbols = if field.attrs.symbols.is_empty() {
        let implicit = &field.attrs.implicit;
        let ast_implicit = &meta.attrs.implicit;
        let symbol = field.ident.to_string();
        let symbol = implicit
            .rename
            .or(ast_implicit.rename)
            .apply_to_field(&symbol);

        let spec = attr::SymbolSpec {
            name: attr::LitString::String(symbol),
            debug: implicit.debug.or(ast_implicit.debug).unwrap_or(false),
        };
        &[spec]
    } else {
        field.attrs.symbols.as_slice()
    };
    symbols
        .iter()
        .map(|spec| {
            let symbol = match &spec.name {
                attr::LitString::String(name) => {
                    let mut symbol = name.to_owned();
                    if let Some(prefix) = &meta.attrs.symbol.prefix {
                        symbol = format!("{}{}", prefix.trim(), symbol);
                    }
                    if let Some(suffix) = &meta.attrs.symbol.suffix {
                        symbol = format!("{}{}", symbol, suffix.trim());
                    }
                    unsafe { CString::from_vec_unchecked(symbol.into_bytes()) }
                }
                attr::LitString::CString(name) => name.to_owned(),
            };
            let logger = field.attrs.logger.unwrap_or(meta.attrs.logger);
            (symbol, spec.debug, logger)
        })
        .map(|(symbol, debug, logger)| {
            let ident = format_ident!("{}", if debug { "debug_symbol" } else { "symbol" });
            let mut expr = quote! {
                lib.#ident(#symbol, None)
            };
            if logger {
                expr.extend(quote! {
                    .inspect(|symbol| {
                        ::log::trace!("Symbol `{}` loaded at {:p}", #symbol.to_string_lossy(), *symbol)
                    })
                    .inspect_err(|e| ::log::warn!("{}", e))
                });
            }
            expr
        })
        .reduce(|acc, expr| quote!(#acc.or_else(|_| #expr)))
        .unwrap()
}

fn generate_symbols_wrapper(cx: &Ctxt, meta: &ast::Metadata) -> TokenStream {
    let mut tokens = TokenStream::new();
    for field in meta.fields.iter() {
        let wrapper = match skip_type_group(field.ty) {
            syn::Type::BareFn(ty) => generate_function_wrapper(&field.ident, ty),
            syn::Type::Reference(ty) => Some(generate_reference_wrapper(&field.ident, ty)),
            syn::Type::Ptr(_) => None,
            syn::Type::Path(ty) => match get_option_inner_type(ty) {
                Some(syn::Type::BareFn(ty)) => generate_optional_function_wrapper(&field.ident, ty),
                Some(syn::Type::Reference(ty)) => {
                    Some(generate_optional_reference_wrapper(&field.ident, ty))
                }
                Some(syn::Type::Ptr(_)) => None,
                _ => {
                    cx.error_spanned_by(
                        field.ty,
                        format!("unsupported type: `{}`", field.ty.into_token_stream()),
                    );
                    None
                }
            },
            _ => {
                cx.error_spanned_by(
                    field.ty,
                    format!("unsupported type: `{}`", field.ty.into_token_stream()),
                );
                None
            }
        };
        if let Some(wrapper) = wrapper {
            tokens.extend(wrapper);
        }
    }
    tokens
}

fn generate_function_wrapper(ident: &syn::Ident, fn_ty: &syn::TypeBareFn) -> Option<TokenStream> {
    match fn_ty.variadic {
        None => {
            let return_type = &fn_ty.output;
            let unsafety = fn_ty.unsafety;
            let args_name = get_bera_fn_arg_idents(fn_ty);
            let args_type = fn_ty.inputs.iter().map(|arg| &arg.ty);
            Some(quote! {
                #[inline]
                pub #unsafety fn #ident(&self, #(#args_name: #args_type),*) #return_type {
                    #unsafety { (self.#ident)(#(#args_name),*) }
                }
            })
        }
        Some(_) => None,
    }
}

fn generate_reference_wrapper(ident: &syn::Ident, ref_ty: &syn::TypeReference) -> TokenStream {
    let ty = &ref_ty.elem;
    let mut_acc = ref_ty.mutability.map(|_| {
        let mut_ident = format_ident!("mut_{ident}");
        quote! {
            #[inline]
            pub fn #mut_ident(&mut self) -> &mut #ty {
                self.#ident
            }
        }
    });
    quote! {
        #[inline]
        pub fn #ident(&self) -> & #ty {
            self.#ident
        }
        #mut_acc
    }
}

fn generate_optional_function_wrapper(
    ident: &syn::Ident,
    fn_ty: &syn::TypeBareFn,
) -> Option<TokenStream> {
    match fn_ty.variadic {
        None => {
            let return_type = match &fn_ty.output {
                syn::ReturnType::Default => quote!(-> ::core::option::Option<()>),
                syn::ReturnType::Type(_, ty) => quote!(-> ::core::option::Option<#ty>),
            };
            let unsafety = fn_ty.unsafety;
            let args_name = get_bera_fn_arg_idents(fn_ty);
            let args_type = fn_ty.inputs.iter().map(|arg| &arg.ty);
            let has_ident = format_ident!("has_{ident}");
            Some(quote! {
                #[inline]
                pub #unsafety fn #ident(&self, #(#args_name: #args_type),*) #return_type {
                    #unsafety { self.#ident.map(|f| f(#(#args_name),*)) }
                }
                #[inline]
                pub fn #has_ident(&self) -> bool {
                    self.#ident.is_some()
                }
            })
        }
        Some(_) => None,
    }
}

fn generate_optional_reference_wrapper(
    ident: &syn::Ident,
    ref_ty: &syn::TypeReference,
) -> TokenStream {
    let ty = &ref_ty.elem;
    match ref_ty.mutability {
        Some(_) => quote! {
            #[inline]
            pub fn #ident(&mut self) -> ::core::option::Option<&mut #ty> {
                if let Some(&mut ref mut val) = self.#ident {
                    Some(val)
                } else {
                    None
                }
            }
        },
        None => quote! {
            #[inline]
            pub fn #ident(&self) -> ::core::option::Option<& #ty> {
                self.#ident
            }
        },
    }
}

fn get_bera_fn_arg_idents(fn_ty: &syn::TypeBareFn) -> Vec<syn::Ident> {
    let args = fn_ty.inputs.iter().enumerate();
    let args = args.map(|(i, arg)| match &arg.name {
        None => format_ident!("arg{i}"),
        Some((ident, _)) => ident.to_owned(),
    });
    args.collect()
}

fn skip_type_group(ty: &syn::Type) -> &syn::Type {
    match ty {
        syn::Type::Group(group) => skip_type_group(&group.elem),
        _ => ty,
    }
}

fn get_option_inner_type(ty: &syn::TypePath) -> Option<&syn::Type> {
    ty.path
        .segments
        .last()
        .filter(|segment| segment.ident == "Option")
        .map(|segment| &segment.arguments)
        .and_then(|args| match args {
            syn::PathArguments::AngleBracketed(generics) => match generics.args.len() {
                1 => generics.args.first(),
                _ => None,
            },
            _ => None,
        })
        .and_then(|generic| match generic {
            syn::GenericArgument::Type(ty) => Some(ty),
            _ => None,
        })
}
