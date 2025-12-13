use crate::internals::Ctxt;
use crate::internals::case::RenameRule;
use quote::ToTokens;
use std::ffi::CString;
use syn::spanned::Spanned;
use syn::{Error, Result};

#[derive(Default)]
pub struct Metadata {
    pub implicit: ImplicitMetadata,
    pub symbol: SymbolMetadata,
    pub logger: bool,
}

#[derive(Default)]
pub struct ImplicitMetadata {
    pub rename: RenameRule,
    pub debug: Option<bool>,
}

#[derive(Default)]
pub struct SymbolMetadata {
    pub prefix: Option<String>,
    pub suffix: Option<String>,
}

impl Metadata {
    pub fn from_ast(cx: &Ctxt, ast: &syn::DeriveInput) -> Self {
        let mut metadata = Metadata::default();
        for attr in ast.attrs.iter() {
            if !attr.path().is_ident("native") {
                continue;
            }
            if let Err(err) = attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("implicit") {
                    metadata.implicit = ImplicitMetadata::from_ast(cx, &meta);
                } else if meta.path.is_ident("symbol") {
                    metadata.symbol = SymbolMetadata::from_ast(cx, &meta);
                } else if meta.path.is_ident("logger") {
                    if let Some(lit) = get_lit_bool(cx, "logger", &meta)? {
                        metadata.logger = lit.value;
                    }
                } else {
                    let path = meta.path.to_token_stream().to_string().replace(' ', "");
                    return Err(meta.error(format_args!("unknown attribute `{}`", path)));
                }
                Ok(())
            }) {
                cx.syn_error(err);
            }
        }
        metadata
    }
}

impl ImplicitMetadata {
    fn from_ast(cx: &Ctxt, meta: &syn::meta::ParseNestedMeta) -> Self {
        let mut metadata = Self::default();
        if let Err(err) = meta.parse_nested_meta(|meta| {
            if meta.path.is_ident("rename") {
                if let Some(lit) = get_lit_str(cx, "rename", &meta)? {
                    match RenameRule::from_str(lit.value().as_str()) {
                        Ok(rule) => metadata.rename = rule,
                        Err(err) => cx.error_spanned_by(lit, err),
                    }
                }
            } else if meta.path.is_ident("debug") {
                if let Some(lit) = get_lit_bool(cx, "debug", &meta)? {
                    metadata.debug = Some(lit.value);
                }
            } else {
                let path = meta.path.to_token_stream().to_string().replace(' ', "");
                return Err(meta.error(format_args!("unknown attribute `{}`", path)));
            }
            Ok(())
        }) {
            cx.syn_error(err);
        }
        metadata
    }
}

impl SymbolMetadata {
    fn from_ast(cx: &Ctxt, meta: &syn::meta::ParseNestedMeta) -> Self {
        let mut metadata = Self::default();
        if let Err(err) = meta.parse_nested_meta(|meta| {
            if meta.path.is_ident("prefix") {
                if let Some(lit) = get_lit_str(cx, "prefix", &meta)?
                    && !lit.value().is_empty()
                {
                    metadata.prefix = Some(lit.value())
                }
            } else if meta.path.is_ident("suffix") {
                if let Some(lit) = get_lit_str(cx, "suffix", &meta)?
                    && !lit.value().is_empty()
                {
                    metadata.suffix = Some(lit.value())
                }
            } else {
                let path = meta.path.to_token_stream().to_string().replace(' ', "");
                return Err(meta.error(format_args!("unknown attribute `{}`", path)));
            }
            Ok(())
        }) {
            cx.syn_error(err);
        }
        metadata
    }
}

#[derive(Default)]
pub struct FieldMetadata {
    pub implicit: ImplicitMetadata,
    pub symbols: Vec<SymbolSpec>,
    pub logger: Option<bool>,
}

pub struct SymbolSpec {
    pub name: LitString,
    pub debug: bool,
}

pub enum LitString {
    String(String),
    CString(CString),
}

impl FieldMetadata {
    pub fn from_ast(cx: &Ctxt, field: &syn::Field) -> Self {
        let mut metadata = Self::default();
        for attr in field.attrs.iter() {
            if !attr.path().is_ident("native") {
                continue;
            }
            if let Err(err) = attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("implicit") {
                    metadata.implicit = ImplicitMetadata::from_ast(cx, &meta);
                } else if meta.path.is_ident("symbol") {
                    metadata.symbols = get_symbol_array(cx, &meta)?
                } else if meta.path.is_ident("logger") {
                    if let Some(lit) = get_lit_bool(cx, "logger", &meta)? {
                        metadata.logger = Some(lit.value);
                    }
                } else {
                    let path = meta.path.to_token_stream().to_string().replace(' ', "");
                    return Err(meta.error(format_args!("unknown attribute `{}`", path)));
                }
                Ok(())
            }) {
                cx.syn_error(err);
            }
        }
        metadata
    }
}

fn skip_expr_group(expr: &syn::Expr) -> &syn::Expr {
    match expr {
        syn::Expr::Group(group) => skip_expr_group(&group.expr),
        _ => expr,
    }
}

fn get_lit_str(
    cx: &Ctxt,
    attr_name: &str,
    meta: &syn::meta::ParseNestedMeta,
) -> Result<Option<syn::LitStr>> {
    let expr: syn::Expr = meta.value()?.parse()?;
    if let syn::Expr::Lit(syn::ExprLit {
        lit: syn::Lit::Str(lit),
        ..
    }) = skip_expr_group(&expr)
    {
        let suffix = lit.suffix();
        if !suffix.is_empty() {
            cx.error_spanned_by(
                lit,
                format!("unexpected suffix `{}` on string literal", suffix),
            );
        }
        Ok(Some(lit.to_owned()))
    } else {
        cx.error_spanned_by(
            expr,
            format!(
                "expected {0} attribute to be a string: `{0} = \"...\"`",
                attr_name,
            ),
        );
        Ok(None)
    }
}

fn get_lit_bool(
    cx: &Ctxt,
    attr_name: &str,
    meta: &syn::meta::ParseNestedMeta,
) -> Result<Option<syn::LitBool>> {
    if !meta.input.peek(syn::Token![=]) {
        Ok(Some(syn::LitBool::new(true, meta.path.span())))
    } else {
        let expr: syn::Expr = meta.value()?.parse()?;
        if let syn::Expr::Lit(syn::ExprLit {
            lit: syn::Lit::Bool(lit),
            ..
        }) = skip_expr_group(&expr)
        {
            Ok(Some(lit.to_owned()))
        } else {
            cx.error_spanned_by(
                expr,
                format!(
                    "expected {0} attribute to be a bool: `{0} = false`",
                    attr_name,
                ),
            );
            Ok(None)
        }
    }
}

fn get_symbol_array(cx: &Ctxt, meta: &syn::meta::ParseNestedMeta) -> Result<Vec<SymbolSpec>> {
    let expr: syn::Expr = meta.value()?.parse()?;
    let expr_array = match expr {
        syn::Expr::Array(syn::ExprArray { elems, .. }) => Ok(elems.into_iter().collect::<Vec<_>>()),
        syn::Expr::Lit(syn::ExprLit {
            lit: syn::Lit::Str(_),
            ..
        }) => Ok(vec![expr]),
        _ => Err(Error::new_spanned(
            expr.into_token_stream(),
            format!(
                "expected {0} attribute to be a string or array of strings: `{0} = \"...\"`or `{0} = [\"...\"]`",
                "symbol",
            ),
        )),
    }?;
    let iterator = expr_array.iter().flat_map(|expr| {
        let has_debug_suffix = |lit: &syn::ExprLit| -> bool {
            match lit.lit.suffix().trim() {
                "" => false,
                "d" | "debug" => true,
                suffix => {
                    cx.error_spanned_by(
                        lit,
                        format!("unexpected suffix `{}` on string literal", suffix),
                    );
                    false
                }
            }
        };
        let spec = match expr {
            syn::Expr::Lit(lit) => match &lit.lit {
                syn::Lit::Str(s) => Some(SymbolSpec {
                    name: LitString::String(s.value()),
                    debug: has_debug_suffix(lit),
                }),
                syn::Lit::CStr(s) => Some(SymbolSpec {
                    name: LitString::CString(s.value()),
                    debug: has_debug_suffix(lit),
                }),
                _ => None,
            },
            _ => None,
        };
        if spec.is_none() {
            cx.error_spanned_by(
                expr,
                "expected string literal in `symbol` array, found invalid element",
            );
        }
        spec
    });
    Ok(iterator.collect())
}
