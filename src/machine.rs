use case::CaseExt;
use syn::export::Span;
use syn::parse::{Parse, ParseStream, Result};
use syn::{Attribute, Ident, ItemEnum};

#[derive(Debug)]
pub struct Machine {
    attributes: Vec<Attribute>,
    data: ItemEnum,
}

impl Parse for Machine {
    fn parse(input: ParseStream) -> Result<Self> {
        let attributes: Vec<Attribute> = input.call(Attribute::parse_outer)?;
        let data: syn::ItemEnum = input.parse()?;

        Ok(Machine { attributes, data })
    }
}

impl Machine {
    pub fn generate(&self) -> (&Ident, syn::export::TokenStream) {
        let Machine {
            attributes,
            data: ast,
        } = self;

        //println!("attributes: {:?}", attributes);
        //println!("ast: {:#?}", ast);

        let machine_name = &ast.ident;
        let variants_names = &ast.variants.iter().map(|v| &v.ident).collect::<Vec<_>>();
        let structs_names = variants_names.clone();

        // define the state enum
        let toks = quote! {
          #(#attributes)*
          pub enum #machine_name {
            Error,
            #(#variants_names(#structs_names)),*
          }
        };

        let mut stream = proc_macro::TokenStream::from(toks);

        // define structs for each state
        for ref variant in ast.variants.iter() {
            let name = &variant.ident;

            let fields = &variant
                .fields
                .iter()
                .map(|f| {
                    let vis = &f.vis;
                    let ident = &f.ident;
                    let ty = &f.ty;

                    quote! {
                      #vis #ident: #ty
                    }
                })
                .collect::<Vec<_>>();

            let toks = quote! {
              #(#attributes)*
              pub struct #name {
                #(#fields),*
              }
            };

            stream.extend(proc_macro::TokenStream::from(toks));
        }

        let methods = &ast
            .variants
            .iter()
            .map(|variant| {
                let fn_name = Ident::new(&variant.ident.to_string().to_snake(), Span::call_site());
                let struct_name = &variant.ident;

                let args = &variant
                    .fields
                    .iter()
                    .map(|f| {
                        let ident = &f.ident;
                        let ty = &f.ty;

                        quote! {
                          #ident: #ty
                        }
                    })
                    .collect::<Vec<_>>();

                let arg_names = &variant.fields.iter().map(|f| &f.ident).collect::<Vec<_>>();

                quote! {
                  pub fn #fn_name(#(#args),*) -> #machine_name {
                    #machine_name::#struct_name(#struct_name {
                      #(#arg_names),*
                    })
                  }
                }
            })
            .collect::<Vec<_>>();

        let toks = quote! {
          impl #machine_name {
            #(#methods)*

            pub fn error() -> #machine_name {
              #machine_name::Error
            }
          }
        };

        stream.extend(proc_macro::TokenStream::from(toks));

        (machine_name, stream)
    }
}
