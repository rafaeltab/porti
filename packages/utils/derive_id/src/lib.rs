use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields, Type};

#[proc_macro_derive(DomainIdentity)]
pub fn domain_identity_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let error_message = format!(
        "DomainIdentity can only be derived for tuple structs with a single `u64` field, like `struct {}(u64);`",
        name
    );

    let valid = if let Data::Struct(data_struct) = &input.data {
        if let Fields::Unnamed(fields) = &data_struct.fields {
            fields.unnamed.len() == 1
                && matches!(fields.unnamed.first().unwrap().ty, Type::Path(ref type_path) if type_path.path.is_ident("u64"))
        } else {
            false
        }
    } else {
        false
    };

    if !valid {
        return syn::Error::new_spanned(name, error_message)
            .to_compile_error()
            .into();
    }

    let expanded = quote! {
        impl #name {
            pub fn to_primitive(&self) -> u64 {
                self.0
            }
        }

        impl Clone for #name {
            fn clone(&self) -> Self {
                Self(self.0)
            }
        }

        impl Copy for #name {}

        impl PartialEq for #name {
            fn eq(&self, other: &Self) -> bool {
                self.0 == other.0
            }
        }

        impl Eq for #name {}

        impl PartialOrd for #name {
            fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
                self.0.partial_cmp(&other.0)
            }
        }

        impl Ord for #name {
            fn cmp(&self, other: &Self) -> std::cmp::Ordering {
                self.0.cmp(&other.0)
            }
        }

        impl std::hash::Hash for #name {
            fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
                self.0.hash(state);
            }
        }

        impl std::fmt::Debug for #name {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(f, "{}({})", stringify!(#name), self.0)
            }
        }

        impl std::fmt::Display for #name {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(f, "{}({})", stringify!(#name), self.0)
            }
        }
    };

    TokenStream::from(expanded)
}
