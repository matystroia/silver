use proc_macro::TokenStream;
use quote::quote;

#[proc_macro_derive(Close)]
pub fn derive_close(item: TokenStream) -> TokenStream {
    let ast: syn::DeriveInput = syn::parse(item).unwrap();
    let name = &ast.ident;
    quote! {
        impl Close for #name {
            fn should_close(&self) -> bool { self.close }
        }
    }
    .into()
}
