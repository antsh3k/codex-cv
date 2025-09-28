use proc_macro::TokenStream;
use quote::quote;
use syn::Attribute;
use syn::DeriveInput;
use syn::Expr;
use syn::Ident;
use syn::LitStr;
use syn::Result;
use syn::Token;
use syn::parse::Parse;
use syn::parse_macro_input;

#[derive(Default)]
struct SubagentArgs {
    name: Option<LitStr>,
    description: Option<LitStr>,
    model: Option<LitStr>,
    tools: Vec<Expr>,
    keywords: Vec<Expr>,
    instructions: Option<Expr>,
}

impl SubagentArgs {
    fn from_attributes(attrs: &[Attribute]) -> Result<Self> {
        let mut args = SubagentArgs::default();
        for attr in attrs.iter().filter(|attr| attr.path().is_ident("subagent")) {
            attr.parse_args_with(|input: syn::parse::ParseStream<'_>| {
                while !input.is_empty() {
                    let key: Ident = input.parse()?;
                    input.parse::<Token![=]>()?;
                    match key.to_string().as_str() {
                        "name" => {
                            let lit: LitStr = input.parse()?;
                            args.name = Some(lit);
                        }
                        "description" => {
                            let lit: LitStr = input.parse()?;
                            args.description = Some(lit);
                        }
                        "model" => {
                            let lit: LitStr = input.parse()?;
                            args.model = Some(lit);
                        }
                        "instructions" => {
                            let expr: Expr = input.parse()?;
                            args.instructions = Some(expr);
                        }
                        "tools" => {
                            let content;
                            syn::bracketed!(content in input);
                            let elems = content.parse_terminated(Expr::parse, Token![,])?;
                            args.tools = elems.into_iter().collect();
                        }
                        "keywords" => {
                            let content;
                            syn::bracketed!(content in input);
                            let elems = content.parse_terminated(Expr::parse, Token![,])?;
                            args.keywords = elems.into_iter().collect();
                        }
                        other => {
                            return Err(syn::Error::new(
                                key.span(),
                                format!("unsupported subagent attribute `{other}`"),
                            ));
                        }
                    }

                    if input.peek(Token![,]) {
                        input.parse::<Token![,]>()?;
                    }
                }
                Ok(())
            })?;
        }
        Ok(args)
    }
}

#[proc_macro_derive(Subagent, attributes(subagent))]
pub fn derive_subagent(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let args = match SubagentArgs::from_attributes(&input.attrs) {
        Ok(args) => args,
        Err(err) => return err.to_compile_error().into(),
    };

    let name = match args.name {
        Some(name) => name,
        None => {
            return syn::Error::new_spanned(&input.ident, "missing `name` in #[subagent(...)]")
                .to_compile_error()
                .into();
        }
    };
    let instructions_expr = match args.instructions {
        Some(expr) => expr,
        None => {
            return syn::Error::new_spanned(
                &input.ident,
                "missing `instructions` in #[subagent(...)]",
            )
            .to_compile_error()
            .into();
        }
    };

    let description_tokens = args
        .description
        .as_ref()
        .map(|lit| quote! { .description(Some(#lit.to_string())) })
        .unwrap_or_else(|| quote! { .description(None::<::std::string::String>) });

    let model_tokens = args
        .model
        .as_ref()
        .map(|lit| quote! { .model(Some(#lit.to_string())) })
        .unwrap_or_else(|| quote! { .model(None::<::std::string::String>) });

    let tools_tokens = if args.tools.is_empty() {
        quote! {}
    } else {
        let elems = args.tools.iter();
        quote! { .tools([#(#elems),*]) }
    };

    let keywords_tokens = if args.keywords.is_empty() {
        quote! {}
    } else {
        let elems = args.keywords.iter();
        quote! { .keywords([#(#elems),*]) }
    };

    let ident = &input.ident;

    let builder_tokens = quote! {
        codex_subagents::SubagentBuilder::new(#name)
            #description_tokens
            #model_tokens
            #tools_tokens
            #keywords_tokens
            .instructions(#instructions_expr)
            .source(codex_subagents::AgentSource::Builtin)
            .build()
            .expect("#[derive(Subagent)] produced invalid spec")
    };

    let expanded = quote! {
        impl #ident {
            pub fn subagent_spec() -> codex_subagents::SubagentSpec {
                #builder_tokens
            }
        }

        impl codex_subagents::Subagent for #ident {
            fn spec(&self) -> std::borrow::Cow<'_, codex_subagents::SubagentSpec> {
                std::borrow::Cow::Owned(Self::subagent_spec())
            }
        }
    };

    expanded.into()
}
