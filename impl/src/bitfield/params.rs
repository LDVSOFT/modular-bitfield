use super::config::Config;
use syn::{
    parse::Result,
    spanned::Spanned,
};

/// Raises an unsupported argument compile time error.
fn unsupported_argument<T>(arg: T) -> syn::Error
where
    T: Spanned,
{
    format_err!(arg, "encountered unsupported #[bitfield] attribute")
}

/// The parameters given to the `#[bitfield]` proc. macro.
///
/// # Example
///
/// ```rust
/// # use modular_bitfield::prelude::*;
/// #
/// #[bitfield(bytes = 4, filled = true)]
/// #[derive(BitfieldSpecifier)]
/// pub struct SignedInteger {
///     sign: bool,
///     value: B31,
/// }
/// ```
pub struct ParamArgs {
    args: syn::AttributeArgs,
}

impl syn::parse::Parse for ParamArgs {
    fn parse(input: syn::parse::ParseStream) -> Result<Self> {
        let punctuated =
            <syn::punctuated::Punctuated<_, syn::Token![,]>>::parse_terminated(input)?;
        Ok(Self {
            args: punctuated.into_iter().collect::<Vec<_>>(),
        })
    }
}

impl IntoIterator for ParamArgs {
    type Item = syn::NestedMeta;
    type IntoIter = std::vec::IntoIter<syn::NestedMeta>;

    fn into_iter(self) -> Self::IntoIter {
        self.args.into_iter()
    }
}

impl Config {
    /// Feeds a `bytes: int` parameter to the `#[bitfield]` configuration.
    fn feed_bytes_param(&mut self, name_value: syn::MetaNameValue) -> Result<()> {
        assert!(name_value.path.is_ident("bytes"));
        match &name_value.lit {
            syn::Lit::Int(lit_int) => {
                let span = lit_int.span();
                let value = lit_int.base10_parse::<usize>().map_err(|err| {
                    format_err!(
                        span,
                        "encountered malformatted integer value for bytes parameter: {}",
                        err
                    )
                })?;
                self.bytes(value, name_value.span())?;
            }
            invalid => {
                return Err(format_err!(
                invalid,
                "encountered invalid value argument for #[bitfield] `bytes` parameter",
            ))
            }
        }
        Ok(())
    }

    /// Feeds a `filled: bool` parameter to the `#[bitfield]` configuration.
    fn feed_filled_param(&mut self, name_value: syn::MetaNameValue) -> Result<()> {
        assert!(name_value.path.is_ident("filled"));
        match &name_value.lit {
            syn::Lit::Bool(lit_bool) => {
                self.filled(lit_bool.value, name_value.span())?;
            }
            invalid => {
                return Err(format_err!(
                invalid,
                "encountered invalid value argument for #[bitfield] `filled` parameter",
            ))
            }
        }
        Ok(())
    }

    /// Feeds the given parameters to the `#[bitfield]` configuration.
    ///
    /// # Errors
    ///
    /// If a parameter is malformatted, unexpected, duplicate or in conflict.
    pub fn feed_params<'a, P>(&mut self, params: P) -> Result<()>
    where
        P: IntoIterator<Item = syn::NestedMeta> + 'a,
    {
        for nested_meta in params {
            match nested_meta {
                syn::NestedMeta::Meta(meta) => {
                    match meta {
                        syn::Meta::NameValue(name_value) => {
                            if name_value.path.is_ident("bytes") {
                                self.feed_bytes_param(name_value)?;
                            } else if name_value.path.is_ident("filled") {
                                self.feed_filled_param(name_value)?;
                            } else {
                                return Err(unsupported_argument(name_value))
                            }
                        }
                        unsupported => return Err(unsupported_argument(unsupported)),
                    }
                }
                unsupported => return Err(unsupported_argument(unsupported)),
            }
        }
        Ok(())
    }
}