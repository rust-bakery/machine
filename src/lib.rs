extern crate proc_macro;
/*
#[macro_use] mod static_machine;
#[macro_use] mod dynamic_machine;

#[macro_export]
macro_rules! machine(
  ( $($token:tt)* ) => ( static_machine!( $($token)* ); );
);
*/

#[macro_use]
extern crate syn;
#[macro_use]
extern crate quote;
#[macro_use]
extern crate log;

use std::fmt::Debug;
use std::fs::{File, OpenOptions};
use std::io::{Write, Seek};
use std::collections::{HashSet, HashMap};

use syn::parse::{Parse, ParseStream, Result};
//use proc_macro::{Ident, Span};
use quote::ToTokens;
use syn::Ident;
use syn::export::Span;

//use proc_macro::TokenStream;

#[derive(Debug,Clone)]
struct MachineInput {
}

impl Parse for MachineInput {
  fn parse(input: ParseStream) -> Result<Self> {
    //println!("input: {:?}", input);
    panic!();
  }
}

#[proc_macro]
pub fn machine(input: proc_macro::TokenStream) -> syn::export::TokenStream {
    // Construct a string representation of the type definition
    let s = input.to_string();
    println!("got string: {}", s);

    let ast = parse_macro_input!(input as syn::ItemEnum);

    // Build the impl
    let (name, gen) = impl_machine(&ast);

    //println!("generated: {:?}", gen);
    println!("generated: {}", gen);

    let file_name = format!("{}.rs", name.to_string().to_lowercase());
    let mut file = File::create(&file_name).unwrap();
    file.write_all(gen.to_string().as_bytes());
    file.flush();

    gen
}

fn impl_machine(ast: &syn::ItemEnum) -> (&Ident, syn::export::TokenStream) {
    println!("ast: {:#?}", ast);

    let machine_name = &ast.ident;
    let variants_names = &ast.variants.iter().map(|v| &v.ident).collect::<Vec<_>>();
    let structs_names = variants_names.clone();

    // define the state enum
    let toks = quote! {
      #[derive(Clone,Debug,PartialEq)]
      pub enum #machine_name {
        #(#variants_names(#structs_names)),*
      }
    };

    let mut stream = proc_macro::TokenStream::from(toks);

    // define structs for each state
    for ref variant in ast.variants.iter() {
      let name = &variant.ident;

      let fields = &variant.fields.iter().map(|f| {
        let vis = &f.vis;
        let ident = &f.ident;
        let ty = &f.ty;

        quote!{
          #vis #ident: #ty
        }
      }).collect::<Vec<_>>();

      let toks = quote! {
        #[derive(Clone,Debug,PartialEq)]
        pub struct #name {
          #(#fields),*
        }
      };

      stream.extend(proc_macro::TokenStream::from(toks));

      let methods = &variant.fields.iter().map(|f| {
        let ident = &f.ident;
        let ty = &f.ty;
        let mut_ident = Ident::new(&format!("{}_mut", &f.ident.as_ref().unwrap().to_string()), Span::call_site());

        quote!{
          pub fn #ident(&self) -> &#ty {
            &self.#ident
          }

          pub fn #mut_ident(&mut self) -> &mut #ty {
            &mut self.#ident
          }
        }
      }).collect::<Vec<_>>();

      let name = &variant.ident;
      let toks = quote!{
        impl #name {
          #(#methods)*
        }
      };

      stream.extend(proc_macro::TokenStream::from(toks));
    }

    let methods = &ast.variants.iter().map(|variant| {
      let fn_name = Ident::new(&variant.ident.to_string().to_lowercase(), Span::call_site());
      let struct_name = &variant.ident;//Ident::new(variant.ident.to_string().to_lowercase(), Span::call_site());

      let args = &variant.fields.iter().map(|f| {
        let ident = &f.ident;
        let ty = &f.ty;

        quote!{
          #ident: #ty
        }
      }).collect::<Vec<_>>();

      let arg_names = &variant.fields.iter().map(|f| {
        &f.ident
      }).collect::<Vec<_>>();


      quote! {
        pub fn #fn_name(#(#args),*) -> #machine_name {
          #machine_name::#struct_name(#struct_name {
            #(#arg_names),*
          })
        }
      }
    }).collect::<Vec<_>>();


    let mut h = HashMap::new();
    let mut names = Vec::new();

    // field methods on the enum
    for ref variant in ast.variants.iter() {
      let name = &variant.ident;
      names.push(name);

      for ref f in variant.fields.iter() {
        let vis = &f.vis;
        let ident = &f.ident;
        let ty = &f.ty;

        let entry = h.entry((vis, ident, ty)).or_insert(Vec::new());
        entry.push(name);
      }
    }

    let field_methods = h.iter().map(|((vis, ident, ty), field_names)| {
      let mut_ident = Ident::new(&format!("{}_mut", &ident.as_ref().unwrap().to_string()), Span::call_site());

      // TODO handle the case where all variants have the field (do not use an option then)
      let variants = field_names.iter().map(|name| {
        quote! {
          #machine_name::#name(ref v) => Some(v.#ident()),
        }
      }).collect::<Vec<_>>();

      let mut_variants = field_names.iter().map(|name| {
        quote! {
          #machine_name::#name(ref mut v) => Some(v.#mut_ident()),
        }
      }).collect::<Vec<_>>();

      quote!{
        pub fn #ident(&self) -> Option<&#ty> {
          match self {
            #(#variants)*
            _ => None,
          }
        }

        pub fn #mut_ident(&mut self) -> Option<&mut #ty> {
          match self {
            #(#mut_variants)*
            _ => None,
          }
        }
      }

    }).collect::<Vec<_>>();

    let toks = quote!{
      impl #machine_name {
        #(#methods)*
        #(#field_methods)*
      }
    };

    stream.extend(proc_macro::TokenStream::from(toks));

    (machine_name, stream)
}

fn validate(ast: &syn::DeriveInput) {
  /*match ast.body {
    syn::Body::Struct(_) => panic!("machine implementation can only be derived from a struct"),
    _ => {},
  }
*/
}

#[derive(Debug)]
struct Transitions {
  pub machine_name: Ident,
  pub transitions: Vec<Transition>,
}

#[derive(Debug)]
struct Transition {
  pub start: Ident,
  pub message: Ident,
  pub end: Vec<Ident>,
}

impl Parse for Transitions {
  fn parse(input: ParseStream) -> Result<Self> {
    let machine_name: Ident = input.parse()?;
    let _: Token![,] = input.parse()?;

    let content;
    bracketed!(content in input);

    //println!("content: {:?}", content);
    let mut transitions = Vec::new();

    let t: Transition = content.parse()?;
    transitions.push(t);

    loop {
      let lookahead = content.lookahead1();
      if lookahead.peek(Token![,]) {
        let _: Token![,] = content.parse()?;
        let t: Transition = content.parse()?;
        transitions.push(t);
      } else {
        break;
      }
    }

    Ok(Transitions {
      machine_name, transitions,
    })
  }
}

impl Parse for Transition {
  fn parse(input: ParseStream) -> Result<Self> {
    let left;
    parenthesized!(left in input);

    let start: Ident = left.parse()?;
    let _: Token![,] = left.parse()?;
    let message: Ident = left.parse()?;

    let _: Token![=>] = input.parse()?;

    let end = match input.parse::<Ident>() {
      Ok(i) => vec![i],
      Err(_) => {
        let content;
        bracketed!(content in input);

        //println!("content: {:?}", content);
        let mut states = Vec::new();

        let t: Ident = content.parse()?;
        states.push(t);

        loop {
          let lookahead = content.lookahead1();
          if lookahead.peek(Token![,]) {
            let _: Token![,] = content.parse()?;
            let t: Ident = content.parse()?;
            states.push(t);
          } else {
            break;
          }
        }

        states
      }
    };

    Ok(Transition { start, message, end })
  }
}

#[proc_macro]
pub fn transitions(input: proc_macro::TokenStream) -> syn::export::TokenStream {
    println!("\ninput: {:?}", input);
    let mut stream = proc_macro::TokenStream::new();

    let transitions = parse_macro_input!(input as Transitions);
    println!("\nparsed transitions: {:#?}", transitions);

    let machine_name = transitions.machine_name;

    let mut messages = HashMap::new();
    for t in transitions.transitions.iter() {
      let entry = messages.entry(&t.message).or_insert(Vec::new());
      entry.push((&t.start, &t.end));
    }

    // create an enum from the messages
    let message_enum_ident = Ident::new(&format!("{}Messages", &machine_name.to_string()), Span::call_site());
    let variants_names = &messages.keys().collect::<Vec<_>>();
    let structs_names = variants_names.clone();

    // define the state enum
    let toks = quote! {
      #[derive(Clone,Debug,PartialEq)]
      pub enum #message_enum_ident {
        #(#variants_names(#structs_names)),*
      }
    };

    stream.extend(proc_macro::TokenStream::from(toks));

    let functions = messages.iter().map(|(msg, moves)| {
      let fn_ident = Ident::new(&format!("on_{}", &msg.to_string()), Span::call_site());
      let mv = moves.iter().map(|(start, end)| {
        if end.len() == 1 {
          let end_state = &end[0];
          quote!{
            #machine_name::#start(state) => Some(#machine_name::#end_state(state.#fn_ident(input))),
          }
        } else {
          quote!{
            #machine_name::#start(state) => Some(state.#fn_ident(input)),
          }
        }
      }).collect::<Vec<_>>();

      quote! {
        pub fn #fn_ident(self, input: #msg) -> Option<#machine_name> {
          match self {
          #(#mv)*
            _ => None,
          }
        }
      }
    }).collect::<Vec<_>>();

    let toks = quote!{
      impl #machine_name {
        #(#functions)*
      }
    };


    stream.extend(proc_macro::TokenStream::from(toks));

    //println!("generated: {:?}", gen);
    println!("generated transitions: {}", stream);
    let file_name = format!("{}.rs", machine_name.to_string().to_lowercase());
    let mut file = OpenOptions::new().write(true).open(&file_name).unwrap();
    file.seek(std::io::SeekFrom::End(0)).expect("seek");
    file.write_all(stream.to_string().as_bytes()).expect("write_all");
    file.flush();

    stream

}
