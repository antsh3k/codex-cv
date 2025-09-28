use proc_macro::TokenStream;
use quote::quote;
use syn::Data;
use syn::DeriveInput;
use syn::Ident;
use syn::LitStr;
use syn::parse_macro_input;
use syn::spanned::Spanned;

#[proc_macro_derive(Subagent, attributes(subagent))]
pub fn derive_subagent(item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);

    let spec_field = match parse_spec_field(&input) {
        Ok(field) => field,
        Err(err) => return err.into_compile_error().into(),
    };

    let ident = &input.ident;

    let expanded = quote! {
        impl ::codex_subagents::Subagent for #ident {
            fn spec(&self) -> &::codex_subagents::SubagentSpec {
                &self.#spec_field
            }

            fn spec_mut(&mut self) -> &mut ::codex_subagents::SubagentSpec {
                &mut self.#spec_field
            }
        }
    };

    expanded.into()
}

fn parse_spec_field(input: &DeriveInput) -> Result<Ident, syn::Error> {
    let default = Ident::new("spec", input.span());
    let mut name = default.clone();

    for attr in &input.attrs {
        if !attr.path().is_ident("subagent") {
            continue;
        }

        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("spec") {
                let lit: LitStr = meta.value()?.parse()?;
                name = Ident::new(&lit.value(), lit.span());
                Ok(())
            } else {
                Err(meta.error("unsupported attribute for #[subagent]"))
            }
        })?;
    }

    match &input.data {
        Data::Struct(data_struct) => {
            for field in &data_struct.fields {
                if let Some(field_ident) = &field.ident {
                    if field_ident == &name {
                        return Ok(name);
                    }
                }
            }
            Err(syn::Error::new(
                name.span(),
                format!("field `{}` not found on struct", name),
            ))
        }
        _ => Err(syn::Error::new(
            input.span(),
            "#[derive(Subagent)] can only be used with structs",
        )),
    }
}
