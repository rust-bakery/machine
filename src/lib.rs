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
use syn::{Abi, FnArg, FnDecl, Generics, ItemFn, Ident, MethodSig, ReturnType, Type, Visibility, WhereClause};
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

    let toks = quote!{
      impl #machine_name {
        #(#methods)*
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

#[proc_macro]
pub fn methods(input: proc_macro::TokenStream) -> syn::export::TokenStream {
    println!("\ninput: {:?}", input);
    let mut stream = proc_macro::TokenStream::new();

    let methods = parse_macro_input!(input as Methods);
    println!("\nparsed methods: {:#?}", methods);

    let mut h = HashMap::new();
    for method in methods.methods.iter() {
      for state in method.states.iter() {
        let entry = h.entry(state).or_insert(Vec::new());
        entry.push(&method.method_type);
      }
    }

    for (state, methods) in h.iter() {
      let method_toks = methods.iter().map(|method| {
        match method {
          MethodType::Get(ident, ty) => {
            quote!{
              pub fn #ident(&self) -> &#ty {
                &self.#ident
              }
            }
          },
          MethodType::Set(ident, ty) => {
            let mut_ident = Ident::new(&format!("{}_mut", &ident.to_string()), Span::call_site());
            quote!{
              pub fn #mut_ident(&mut self) -> &mut #ty {
                &mut self.#ident
              }
            }
          }
          MethodType::Fn(m) => {
            // we let the user implement these methods on the types
            quote!{}
          }
        }
      }).collect::<Vec<_>>();

      let toks = quote!{
        impl #state {
          #(#method_toks)*
        }
      };

      stream.extend(proc_macro::TokenStream::from(toks));
    };

    let machine_name = methods.machine_name;
    let wrapper_methods = methods.methods.iter().map(|method| {
      match &method.method_type {
        MethodType::Get(ident, ty) => {
          let variants = method.states.iter().map(|state| {
            quote! {
              #machine_name::#state(ref v) => Some(v.#ident()),
            }
          }).collect::<Vec<_>>();
          quote!{
            pub fn #ident(&self) -> Option<&#ty> {
              match self {
                #(#variants)*
                _ => None,
              }
            }
          }
        },
        MethodType::Set(ident, ty) => {
          let mut_ident = Ident::new(&format!("{}_mut", &ident.to_string()), Span::call_site());

          let variants = method.states.iter().map(|state| {
            quote! {
              #machine_name::#state(ref mut v) => Some(v.#mut_ident()),
            }
          }).collect::<Vec<_>>();
          quote!{
            pub fn #mut_ident(&mut self) -> Option<&mut #ty> {
              match self {
                #(#variants)*
                _ => None,
              }
            }
          }
        },
        MethodType::Fn(m) => {
          let ident = &m.ident;
          let args = m.decl.inputs.iter().filter(|arg| {
            match arg {
              FnArg::Captured(_) => true,
              _ => false,
            }
          }).map(|arg| {
            if let FnArg::Captured(a) = arg {
              &a.pat
            } else {
              panic!();
            }
          }).collect::<Vec<_>>();

          let variants = method.states.iter().map(|state| {
            let a = args.clone();
            quote! {
              #machine_name::#state(ref v) => Some(v.#ident( #(#a),* )),
            }
          }).collect::<Vec<_>>();

          let inputs = &m.decl.inputs;
          let output = match &m.decl.output {
            ReturnType::Default => quote!{},
            ReturnType::Type(arrow, ty) => quote!{
              #arrow Option<#ty>
            },
          };

          quote!{
            pub fn #ident(#inputs) #output {
              match self {
                #(#variants)*
                _ => None,
              }
            }
          }
        }
      }
    }).collect::<Vec<_>>();

    let toks = quote!{
      impl #machine_name {
        #(#wrapper_methods)*
      }
    };

    stream.extend(proc_macro::TokenStream::from(toks));

    let file_name = format!("{}.rs", machine_name.to_string().to_lowercase());
    let mut file = OpenOptions::new().write(true).open(&file_name).unwrap();
    file.seek(std::io::SeekFrom::End(0)).expect("seek");
    file.write_all(stream.to_string().as_bytes()).expect("write_all");
    file.flush();

    stream
}

#[derive(Debug)]
struct Methods {
  pub machine_name: Ident,
  pub methods: Vec<Method>,
}

#[derive(Debug)]
struct Method {
  pub states: Vec<Ident>,
  pub method_type: MethodType,
}
#[derive(Debug)]
enum MethodType {
  Get(Ident, Type),
  Set(Ident, Type),
  Fn(MethodSig),
}

impl Parse for Methods {
  fn parse(input: ParseStream) -> Result<Self> {
    let machine_name: Ident = input.parse()?;
    let _: Token![,] = input.parse()?;

    let content;
    bracketed!(content in input);

    let mut methods = Vec::new();

    let t: Method = content.parse()?;
    methods.push(t);

    loop {
      let lookahead = content.lookahead1();
      if lookahead.peek(Token![,]) {
        let _: Token![,] = content.parse()?;
        let t: Method = content.parse()?;
        methods.push(t);
      } else {
        break;
      }
    }

    Ok(Methods {
      machine_name, methods,
    })
  }
}

impl Parse for Method {
  fn parse(input: ParseStream) -> Result<Self> {
    let mut states = Vec::new();

    let state: Ident = input.parse()?;
    states.push(state);

    loop {
      let lookahead = input.lookahead1();
      if lookahead.peek(Token![,]) {
        let _: Token![,] = input.parse()?;
        let state: Ident = input.parse()?;
        states.push(state);
      } else {
        break;
      }
    }

    let _: Token![=>] = input.parse()?;
    let method_type = match parse_method_sig(input) {
      Ok(f) => MethodType::Fn(f),
      Err(e) => {
        let i: Ident = input.parse()?;
        let name: Ident = input.parse()?;
        let _: Token![:] = input.parse()?;
        let ty: Type = input.parse()?;

        if i.to_string() == "get" {
          MethodType::Get(name, ty)
        } else if i.to_string() == "set" {
          MethodType::Set(name, ty)
        } else {
          return Err(syn::Error::new(i.span(), "expected `get` or `set`"));
        }
      }
    };

    Ok(Method { states, method_type })
  }
}

fn parse_method_sig(input: ParseStream) -> Result<MethodSig> {
  let vis: Visibility = input.parse()?;
  let constness: Option<Token![const]> = input.parse()?;
  let unsafety: Option<Token![unsafe]> = input.parse()?;
  let asyncness: Option<Token![async]> = input.parse()?;
  let abi: Option<Abi> = input.parse()?;
  let fn_token: Token![fn] = input.parse()?;
  let ident: Ident = input.parse()?;
  let generics: Generics = input.parse()?;

  let content;
  let paren_token = parenthesized!(content in input);
  let inputs = content.parse_terminated(FnArg::parse)?;

  let output: ReturnType = input.parse()?;
  let where_clause: Option<WhereClause> = input.parse()?;

  Ok(MethodSig {
    constness, unsafety, asyncness, abi, ident,
    decl: FnDecl {
      fn_token: fn_token,
      paren_token: paren_token,
      inputs: inputs,
      output: output,
      variadic: None,
      generics: Generics {
        where_clause: where_clause,
        ..generics
      },
    }
  })

}
