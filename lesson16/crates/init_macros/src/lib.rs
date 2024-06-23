use proc_macro::TokenStream;
use quote::quote;
use syn::parenthesized;
use syn::parse::{Parse, ParseStream, Result};
use syn::{parse_macro_input, punctuated::Punctuated, Ident, Token};

struct EnumInitInput {
    enum_name: Ident,
    variants: Punctuated<Ident, Token![,]>,
}

impl Parse for EnumInitInput {
    fn parse(input: ParseStream) -> Result<Self> {
        let enum_name = input.parse()?;
        let _: Token![,] = input.parse()?;
        let variants = Punctuated::parse_terminated(input)?;
        Ok(EnumInitInput {
            enum_name,
            variants,
        })
    }
}

/// Create shorter inits for enums without fields
///
/// # Example
///
/// ```
/// create_enum_init_functions!(DBError, UserNotFound, ConnectionError, UserInsertionError);
/// ```
/// generates
/// ```
/// pub fn user_not_found() -> DBError {
///     DBError::UserNotFound
/// }
/// pub fn connection_error() -> DBError {
///     DBError::ConnectionError
/// }
/// pub fn user_insertion_error() -> DBError {
///     DBError::UserInsertionError
/// }
#[proc_macro]
pub fn create_enum_init_functions(input: TokenStream) -> TokenStream {
    let EnumInitInput {
        enum_name,
        variants,
    } = parse_macro_input!(input as EnumInitInput);

    let mut functions = Vec::new();

    for variant in variants {
        let variant_str = variant.to_string();
        let snake_case_name =
            convert_case::Casing::to_case(&variant_str, convert_case::Case::Snake);
        let func_ident = Ident::new(&snake_case_name, variant.span());

        functions.push(quote! {
            pub fn #func_ident() -> #enum_name {
                #enum_name::#variant
            }
        });
    }

    TokenStream::from(quote! {
            #(#functions)*
    })
}

struct Variant {
    ident: Ident,
    types: Punctuated<Ident, Token![,]>,
}

impl Parse for Variant {
    fn parse(input: ParseStream) -> Result<Self> {
        let ident = input.parse()?;
        let content;
        parenthesized!(content in input);
        let types = Punctuated::parse_terminated(&content)?;
        Ok(Variant { ident, types })
    }
}

struct EnumInitInput2 {
    enum_name: Ident,
    variants: Punctuated<Variant, Token![,]>,
}

impl Parse for EnumInitInput2 {
    fn parse(input: ParseStream) -> Result<Self> {
        let enum_name = input.parse()?;
        let _: Token![,] = input.parse()?;
        let variants = Punctuated::parse_terminated(input)?;
        Ok(EnumInitInput2 {
            enum_name,
            variants,
        })
    }
}

/// Create shorter inits for enums with variants that have fields
///
/// # Example
///
/// ```
/// create_valueenum_init_functions!(
///     ServerResponse,
///     Message(MessageResponse),
///     AuthToken(String),
///     Error(ErrorResponse)
/// );
/// ```
/// generates
/// ```
/// pub fn message(message_response) -> ServerResponse {
///     ServerResponse::Message(message_response)
/// }
/// pub fn auth_token(auth_token) -> ServerResponse {
///     ServerResponse::AuthToken(auth_token)
/// }
/// pub fn error(error_response) -> ServerResponse {
///     ServerResponse::Error(error_response)
/// }
/// ```
#[proc_macro]
pub fn create_valueenum_init_functions(input: TokenStream) -> TokenStream {
    let EnumInitInput2 {
        enum_name,
        variants,
    } = parse_macro_input!(input as EnumInitInput2);

    let mut functions = Vec::new();

    for variant in variants {
        let variant_ident = &variant.ident;
        let variant_str = variant_ident.to_string();
        let snake_case_name =
            convert_case::Casing::to_case(&variant_str, convert_case::Case::Snake);
        let func_ident = Ident::new(&snake_case_name, variant_ident.span());

        if !variant.types.is_empty() {
            let mut args = Vec::new();
            let params = &variant.types;
            for (index, _) in params.iter().enumerate() {
                let arg = Ident::new(&format!("arg{}", index), variant_ident.span());
                args.push(arg);
            }

            functions.push(quote! {
                pub fn #func_ident(#(#args: #params),*) -> #enum_name {
                    #enum_name::#variant_ident(#(#args),*)
                }
            });
        } else {
            functions.push(quote! {
                pub fn #func_ident() -> #enum_name {
                    #enum_name::#variant_ident
                }
            });
        }
    }

    TokenStream::from(quote! {
        #(#functions)*
    })
}
