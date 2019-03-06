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
    let machine_name = &self.data.ident;
    let mut stream = proc_macro::TokenStream::new();

    stream.extend(self.generate_enum());
    stream.extend(self.generate_variants());
    stream.extend(self.generate_impl());

    (machine_name, stream)
  }

  fn generate_enum(&self) -> syn::export::TokenStream {
    let Machine {
      attributes,
      data: ast,
    } = self;

    let machine_name = &ast.ident;
    let variants_names = &ast.variants.iter().map(|v| &v.ident).collect::<Vec<_>>();
    let structs_names = variants_names.clone();

    // define the state enum
    let tokens = quote! {
      #(#attributes)*
      pub enum #machine_name {
        Error,
        #(#variants_names(#structs_names)),*
      }
    };

    proc_macro::TokenStream::from(tokens)
  }

  fn generate_variants(&self) -> syn::export::TokenStream {
    let Machine {
      attributes,
      data: ast,
    } = self;

    let mut stream = proc_macro::TokenStream::new();

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

      let tokens = quote! {
        #(#attributes)*
        pub struct #name {
          #(#fields),*
        }
      };

      stream.extend(proc_macro::TokenStream::from(tokens));
    }

    stream
  }

  fn generate_impl(&self) -> syn::export::TokenStream {
    let ast = &self.data;
    let machine_name = &ast.ident;

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

    let tokens = quote! {
      impl #machine_name {
        #(#methods)*

        pub fn error() -> #machine_name {
          #machine_name::Error
        }
      }
    };

    proc_macro::TokenStream::from(tokens)
  }
}
