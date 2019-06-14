//! # Machine
//!
//! ## Features
//!
//! This crate defines three procedural macros to help you write enum based state machines,
//! without writing the associated boilerplate.
//!
//! * define a state machine as an enum, each variant can contain members
//! * an Error state for invalid transitions is added automatically
//! * transitions can have multiple end states if needed (conditions depending on message content, etc)
//! * accessors can be generated for state members
//! * wrapper methods and accessors are generated on the parent enum
//! * the generated code is also written in the `target/machine` directory for further inspection
//! * a dot file is written in the `target/machine` directory for graph generation
//!
//! ## Usage
//!
//! machine is available on [crates.io](https://crates.io/crates/machine) and can be included in your Cargo enabled project like this:
//!
//! ```toml
//! [dependencies]
//! machine = "^0.2"
//! ```
//!
//! Then include it in your code like this:
//!
//! ```rust,ignore
//! #[macro_use]
//! extern crate machine;
//! ```
//!
//! ## Example: the traffic light
//!
//! We'll define a state machine representing a traffic light, specifying a maximum
//! number of cars passing while in the green state.
//!
//! The following machine definition:
//!
//! ```rust,ignore
//! machine!(
//!   enum Traffic {
//!     Green { count: u8 },
//!     Orange,
//!     Red
//!   }
//! );
//! ```
//!
//! will produce the following code:
//!
//! ```rust,ignore
//! #[derive(Clone, Debug, PartialEq)]
//! pub enum Traffic {
//!     Error,
//!     Green(Green),
//!     Orange(Orange),
//!     Red(Red),
//! }
//!
//! #[derive(Clone, Debug, PartialEq)]
//! pub struct Green {
//!     count: u8,
//! }
//!
//! #[derive(Clone, Debug, PartialEq)]
//! pub struct Orange {}
//!
//! #[derive(Clone, Debug, PartialEq)]
//! pub struct Red {}
//!
//! impl Traffic {
//!   pub fn green(count: u8) -> Traffic {
//!     Traffic::Green(Green { count })
//!   }
//!   pub fn orange() -> Traffic {
//!     Traffic::Orange(Orange {})
//!   }
//!   pub fn red() -> Traffic {
//!     Traffic::Red(Red {})
//!   }
//!   pub fn error() -> Traffic {
//!     Traffic::Error
//!   }
//! }
//! ```
//!
//! ### Transitions
//!
//! From there, we can define the `Advance` message to go to the next color, and the associated
//! transitions:
//!
//! ```rust,ignore
//! #[derive(Clone,Debug,PartialEq)]
//! pub struct Advance;
//!
//! transitions!(Traffic,
//!   [
//!     (Green, Advance) => Orange,
//!     (Orange, Advance) => Red,
//!     (Red, Advance) => Green
//!   ]
//! );
//! ```
//!
//! This will generate an enum holding the messages for that state machine,
//! and a `on_advance` method on the parent enum.
//!
//! ```rust,ignore
//! #[derive(Clone, Debug, PartialEq)]
//! pub enum TrafficMessages {
//!     Advance(Advance),
//! }
//!
//! impl Traffic {
//!   pub fn on_advance(self, input: Advance) -> Traffic {
//!     match self {
//!       Traffic::Green(state) => Traffic::Orange(state.on_advance(input)),
//!       Traffic::Orange(state) => Traffic::Red(state.on_advance(input)),
//!       Traffic::Red(state) => Traffic::Green(state.on_advance(input)),
//!       _ => Traffic::Error,
//!     }
//!   }
//! }
//! ```
//!
//! The compiler will then complain that the `on_advance` is missing on the
//! `Green`, `Orange` and `Red` structures:
//!
//! ```text,ignore
//! error[E0599]: no method named on_advance found for type Green in the current scope
//!   --> tests/t.rs:18:1
//!    |
//! 4  | / machine!(
//! 5  | |   enum Traffic {
//! 6  | |     Green { count: u8 },
//! 7  | |     Orange,
//! 8  | |     Red,
//! 9  | |   }
//! 10 | | );
//!    | |__- method `on_advance` not found for this
//! ...
//! 18 | / transitions!(Traffic,
//! 19 | |   [
//! 20 | |     (Green, Advance) => Orange,
//! 21 | |     (Orange, Advance) => Red,
//! 22 | |     (Red, Advance) => Green
//! 23 | |   ]
//! 24 | | );
//!    | |__^
//!
//! [...]
//! ```
//!
//! The `transitions` macro takes care of the boilerplate, writing the wrapper
//! methods, and making sure that a state machine receiving the wrong message
//! will get into the error state. But we still need to define manually the
//! transition functions for each of our states, since most of the work will
//! be done there:
//!
//! ```rust,ignore
//! impl Green {
//!   pub fn on_advance(self, _: Advance) -> Orange {
//!     Orange {}
//!   }
//! }
//!
//! impl Orange {
//!   pub fn on_advance(self, _: Advance) -> Red {
//!     Red {}
//!   }
//! }
//!
//! impl Red {
//!   pub fn on_advance(self, _: Advance) -> Green {
//!     Green {
//!       count: 0
//!     }
//!   }
//! }
//! ```
//!
//! Now we want to add a message to count passing cars when in the green state,
//! and switch to the orange state if at least 10 cars have passed.
//! So the `PassCar` message is only accepted by the green state, and the
//! transition has two possible end states, green and orange.
//! While we might want a clean state machine where each state and message
//! combination only has one end state, we could have conditions depending
//! on message values, or state members that would not require creating
//! new states or messages instead:
//!
//! ```rust,ignore
//! #[derive(Clone,Debug,PartialEq)]
//! pub struct PassCar { count: u8 }
//!
//! transitions!(Traffic,
//!   [
//!     (Green, Advance) => Orange,
//!     (Orange, Advance) => Red,
//!     (Red, Advance) => Green,
//!     (Green, PassCar) => [Green, Orange]
//!   ]
//! );
//!
//! impl Green {
//!   pub fn on_pass_car(self, input: PassCar) -> Traffic {
//!     let count = self.count + input.count;
//!     if count >= 10 {
//!       println!("reached max cars count: {}", count);
//!       Traffic::orange()
//!     } else {
//!       Traffic::green(count)
//!     }
//!   }
//! }
//! ```
//!
//! The `on_pass_car` method can have multiple end states, so it must
//! return a `Traffic`.
//!
//! The generated code will now contain a `on_pass_car` for the
//! `Traffic` enum. Note that if a state other than `Green`
//! receives the `PassCar` message, the state machine will go
//! into the `Error` state and stay there indefinitely.
//!
//! ```rust,ignore
//! #[derive(Clone, Debug, PartialEq)]
//! pub enum TrafficMessages {
//!   Advance(Advance),
//!   PassCar(PassCar),
//! }
//!
//! impl Traffic {
//!   pub fn on_advance(self, input: Advance) -> Traffic {
//!     match self {
//!       Traffic::Green(state) => Traffic::Orange(state.on_advance(input)),
//!       Traffic::Orange(state) => Traffic::Red(state.on_advance(input)),
//!       Traffic::Red(state) => Traffic::Green(state.on_advance(input)),
//!       _ => Traffic::Error,
//!     }
//!   }
//!
//!   pub fn on_pass_car(self, input: PassCar) -> Traffic {
//!     match self {
//!       Traffic::Green(state) => state.on_pass_car(input),
//!       _ => Traffic::Error,
//!     }
//!   }
//! }
//! ```
//!
//! The complete generated code can be found in `target/machine/traffic.rs`.
//!
//! The machine crate will also generate the `target/machine/traffic.dot` file
//! for graphviz usage:
//!
//! ```dot
//! digraph Traffic {
//! Green -> Orange [ label = "Advance" ];
//! Orange -> Red [ label = "Advance" ];
//! Red -> Green [ label = "Advance" ];
//! Green -> Green [ label = "PassCar" ];
//! Green -> Orange [ label = "PassCar" ];
//! }
//! ```
//!
//! `dot -Tpng target/machine/traffic.dot > traffic.png` will generate the following image:
//!
//! ![traffic light transitions graph](https://raw.githubusercontent.com/rust-bakery/machine/master/assets/traffic.png)
//!
//! We can then use the messages to trigger transitions:
//!
//! ```rust,ignore
//! // starting in green state, no cars have passed
//! let mut t = Traffic::Green(Green { count: 0 });
//!
//! t = t.on_pass_car(PassCar { count: 1});
//! t = t.on_pass_car(PassCar { count: 2});
//! // still in green state, 3 cars have passed
//! assert_eq!(t, Traffic::green(3));
//!
//! // each advance call will move to the next color
//! t = t.on_advance(Advance);
//! assert_eq!(t, Traffic::orange());
//!
//! t = t.on_advance(Advance);
//! assert_eq!(t, Traffic::red());
//!
//! t = t.on_advance(Advance);
//! assert_eq!(t, Traffic::green(0));
//! t = t.on_pass_car(PassCar { count: 5 });
//! assert_eq!(t, Traffic::green(5));
//!
//! // when more than 10 cars have passed, go to the orange state
//! t = t.on_pass_car(PassCar { count: 7 });
//! assert_eq!(t, Traffic::orange());
//! t = t.on_advance(Advance);
//! assert_eq!(t, Traffic::red());
//!
//! // if we try to use the PassCar message on state other than Green,
//! // we go into the error state
//! t = t.on_pass_car(PassCar { count: 7 });
//! assert_eq!(t, Traffic::error());
//!
//! // once in the error state, we stay in the error state
//! t = t.on_advance(Advance);
//! assert_eq!(t, Traffic::error());
//! ```
//!
//! ### Methods
//!
//! The `methods!` procedural macro can generate wrapper methods for state member
//! accessors, or require method implementations on states:
//!
//! ```rust,ignore
//! methods!(Traffic,
//!   [
//!     Green => get count: u8,
//!     Green => set count: u8,
//!     Green, Orange, Red => fn can_pass(&self) -> bool
//!   ]
//! );
//! ```
//!
//! This will generate:
//! - a `count()` getter for the `Green` state (`get`) and the wrapping enum
//! - a `count_mut()` setter for the `Green` state (`set`) and the wrapping enum
//! - a `can_pass()` method for the wrapping enum, requiring its implementations for all states
//!
//! Methods can have arguments, and those will be passed to the corresponding method
//! on states, as expected.
//!
//! ```rust,ignore
//! impl Orange {}
//! impl Red {}
//! impl Green {
//!   pub fn count(&self) -> &u8 {
//!     &self.count
//!   }
//!
//!   pub fn count_mut(&mut self) -> &mut u8 {
//!     &mut self.count
//!   }
//! }
//!
//! impl Traffic {
//!   pub fn count(&self) -> Option<&u8> {
//!     match self {
//!       Traffic::Green(ref v) => Some(v.count()),
//!       _ => None,
//!     }
//!   }
//!
//!   pub fn count_mut(&mut self) -> Option<&mut u8> {
//!     match self {
//!       Traffic::Green(ref mut v) => Some(v.count_mut()),
//!       _ => None,
//!     }
//!   }
//!
//!   pub fn can_pass(&self) -> Option<bool> {
//!     match self {
//!       Traffic::Green(ref v) => Some(v.can_pass()),
//!       Traffic::Orange(ref v) => Some(v.can_pass()),
//!       Traffic::Red(ref v) => Some(v.can_pass()),
//!       _ => None,
//!     }
//!   }
//! }
//! ```
//!
//! We can now add the remaining methods and get a working state machine:
//!
//! ```rust,ignore
//! impl Green {
//!   pub fn can_pass(&self) -> bool {
//!     true
//!   }
//! }
//!
//! impl Orange {
//!   pub fn can_pass(&self) -> bool {
//!     false
//!   }
//! }
//!
//! impl Red {
//!   pub fn can_pass(&self) -> bool {
//!     false
//!   }
//! }
//! ```

extern crate case;
extern crate proc_macro;
/*
#[macro_use] mod dynamic_machine;

#[macro_export]
macro_rules! machine(
  ( $($token:tt)* ) => ( static_machine!( $($token)* ); );
);
*/

#[macro_use]
extern crate log;
#[macro_use]
extern crate syn;
#[macro_use]
extern crate quote;

use std::collections::{HashMap, HashSet};
use std::fs::{File, OpenOptions, create_dir};
use std::io::{Seek, Write};

use case::CaseExt;
use syn::export::Span;
use syn::punctuated::Pair;
use syn::parse::{Parse, ParseStream, Result};
use syn::{
    Abi, Attribute, Expr, FnArg, FnDecl, Generics, Ident, ItemEnum, MethodSig, ReturnType, Type,
    WhereClause, PathArguments, GenericArgument,
};
use quote::ToTokens;

struct Machine {
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

#[proc_macro]
pub fn machine(input: proc_macro::TokenStream) -> syn::export::TokenStream {
    let ast = parse_macro_input!(input as Machine);

    // Build the impl
    let (name, gen) = impl_machine(&ast);

    trace!("generated: {}", gen);

    let file_name = format!("target/machine/{}.rs", name.to_string().to_lowercase());
    let _ = create_dir("target/machine");
    File::create(&file_name)
        .and_then(|mut file| {
            file.seek(std::io::SeekFrom::End(0))?;
            file.write_all(gen.to_string().as_bytes())?;
            file.flush()
        })
        .expect("error writing machine definition");

    gen
}

fn impl_machine(m: &Machine) -> (&Ident, syn::export::TokenStream) {
    let Machine { attributes, data } = m;
    let ast = data;
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

#[derive(Debug)]
struct Transitions {
    pub machine_name: Ident,
    pub transitions: Vec<Transition>,
}

#[derive(Debug)]
struct Transition {
    pub start: Ident,
    pub message: Type,
    pub end: Vec<Ident>,
}

impl Parse for Transitions {
    fn parse(input: ParseStream) -> Result<Self> {
        let machine_name: Ident = input.parse()?;
        let _: Token![,] = input.parse()?;

        let content;
        bracketed!(content in input);

        trace!("content: {:?}", content);
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
            machine_name,
            transitions,
        })
    }
}

impl Parse for Transition {
    fn parse(input: ParseStream) -> Result<Self> {
        let left;
        parenthesized!(left in input);

        let start: Ident = left.parse()?;
        let _: Token![,] = left.parse()?;
        let message: Type = left.parse()?;

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

        Ok(Transition {
            start,
            message,
            end,
        })
    }
}

impl Transitions {
    pub fn render(&self) {
        let file_name = format!(
            "target/machine/{}.dot",
            self.machine_name.to_string().to_lowercase()
        );
        let _ = create_dir("target/machine");
        let mut file = File::create(&file_name).expect("error opening dot file");

        file.write_all(format!("digraph {} {{\n", self.machine_name.to_string()).as_bytes())
            .expect("error writing to dot file");

        let mut edges = Vec::new();
        for transition in self.transitions.iter() {
            for state in transition.end.iter() {
                edges.push((&transition.start, &transition.message, state));
            }
        }

        for edge in edges.iter() {
            file.write_all(
                &format!("{} -> {} [ label = \"{}\" ];\n", edge.0, edge.2, edge.1.into_token_stream()).as_bytes(),
            )
            .expect("error writing to dot file");
        }

        file.write_all(&b"}"[..])
            .expect("error writing to dot file");
        file.flush().expect("error flushhing dot file");
    }
}

#[proc_macro]
pub fn transitions(input: proc_macro::TokenStream) -> syn::export::TokenStream {
    //println!("\ninput: {:?}", input);
    let mut stream = proc_macro::TokenStream::new();

    let transitions = parse_macro_input!(input as Transitions);
    trace!("\nparsed transitions: {:#?}", transitions);

    transitions.render();

    let machine_name = transitions.machine_name;

    let mut messages = HashMap::new();
    for t in transitions.transitions.iter() {
        let entry = messages.entry(&t.message).or_insert(Vec::new());
        entry.push((&t.start, &t.end));
    }

    //let mut message_types = transitions.transitions.iter().map(|t| &t.message).collect::<Vec<_>>();

    let mut type_arguments = HashSet::new();
    for t in transitions.transitions.iter() {
      let mut args = type_args(&t.message);
      type_arguments.extend(args.drain());
    }

    let type_arguments = reorder_type_arguments(type_arguments);

    // create an enum from the messages
    let message_enum_ident = Ident::new(
        &format!("{}Messages", &machine_name.to_string()),
        Span::call_site(),
    );
    let structs_names = messages.keys().collect::<Vec<_>>();
    let variants_names = structs_names.iter().map(|t| type_last_ident(*t)).collect::<Vec<_>>();


    let type_arg_toks = if type_arguments.is_empty() {
      quote!{}
    } else {
      quote!{
        < #(#type_arguments),* >
      }
    };

    // define the state enum
    let toks = quote! {
      #[derive(Clone,Debug,PartialEq)]
      pub enum #message_enum_ident #type_arg_toks {
        #(#variants_names(#structs_names)),*
      }
    };

    stream.extend(proc_macro::TokenStream::from(toks));
    let functions = messages
      .iter()
      .map(|(msg, moves)| {
        let fn_ident = Ident::new(
          //&format!("on_{}", &msg.to_string().to_snake()),
          &format!("on_{}", type_to_snake(msg)),
          Span::call_site(),
          );
        let mv = moves.iter().map(|(start, end)| {
          if end.len() == 1 {
            let end_state = &end[0];
            quote!{
              #machine_name::#start(state) => #machine_name::#end_state(state.#fn_ident(input)),
            }
          } else {
            quote!{
              #machine_name::#start(state) => state.#fn_ident(input),
            }
          }
        }).collect::<Vec<_>>();

        let type_arguments = reorder_type_arguments(type_args(msg));
        let type_arg_toks = if type_arguments.is_empty() {
          quote!{}
        } else {
          quote!{
            < #(#type_arguments),* >
          }
        };

        quote! {
          pub fn #fn_ident #type_arg_toks(self, input: #msg) -> #machine_name {
            match self {
              #(#mv)*
              _ => #machine_name::Error,
            }
          }
        }
      })
    .collect::<Vec<_>>();

    let matches = messages
      .keys()
      .map(|msg| {
        let fn_ident = Ident::new(
          //&format!("on_{}", &msg.to_string().to_snake()),
          &format!("on_{}", type_to_snake(msg)),
          Span::call_site(),
          );

          let id = type_last_ident(msg);

          quote!{
            #message_enum_ident::#id(message) => self.#fn_ident(message),
          }

      })
    .collect::<Vec<_>>();

    /*let type_arg_toks = if type_arguments.is_empty() {
      quote!{}
    } else {
      quote!{
        < #(#type_arguments),* >
      }
    };*/

    let execute = quote! {
      pub fn execute #type_arg_toks(self, input: #message_enum_ident #type_arg_toks) -> #machine_name {
        match input {
          #(#matches)*
          _ => #machine_name::Error,
        }
      }
    };

    let toks = quote! {
      impl #machine_name {
        #(#functions)*

        #execute
      }
    };

    stream.extend(proc_macro::TokenStream::from(toks));

    //println!("generated: {:?}", gen);
    trace!("generated transitions: {}", stream);
    let _ = create_dir("target/machine");
    let file_name = format!("target/machine/{}.rs", machine_name.to_string().to_lowercase());
    OpenOptions::new()
        .create(true)
        .write(true)
        .open(&file_name)
        .and_then(|mut file| {
            file.seek(std::io::SeekFrom::End(0))?;
            file.write_all(stream.to_string().as_bytes())?;
            file.flush()
        })
        .expect("error writing transitions");

    stream
}

#[proc_macro]
pub fn methods(input: proc_macro::TokenStream) -> syn::export::TokenStream {
    //println!("\ninput: {:?}", input);
    let mut stream = proc_macro::TokenStream::new();

    let methods = parse_macro_input!(input as Methods);
    trace!("\nparsed methods: {:#?}", methods);

    let mut h = HashMap::new();
    for method in methods.methods.iter() {
        for state in method.states.iter() {
            let entry = h.entry(state).or_insert(Vec::new());
            entry.push(&method.method_type);
        }
    }

    for (state, methods) in h.iter() {
        let method_toks = methods
            .iter()
            .map(|method| {
                match method {
                    MethodType::Get(ident, ty) => {
                        quote! {
                          pub fn #ident(&self) -> &#ty {
                            &self.#ident
                          }
                        }
                    }
                    MethodType::Set(ident, ty) => {
                        let mut_ident =
                            Ident::new(&format!("{}_mut", &ident.to_string()), Span::call_site());
                        quote! {
                          pub fn #mut_ident(&mut self) -> &mut #ty {
                            &mut self.#ident
                          }
                        }
                    }
                    MethodType::Fn(_) => {
                        // we let the user implement these methods on the types
                        quote! {}
                    }
                }
            })
            .collect::<Vec<_>>();

        let toks = quote! {
          impl #state {
            #(#method_toks)*
          }
        };

        stream.extend(proc_macro::TokenStream::from(toks));
    }

    let machine_name = methods.machine_name;
    let wrapper_methods = methods
        .methods
        .iter()
        .map(|method| match &method.method_type {
            MethodType::Get(ident, ty) => {
                let variants = method
                    .states
                    .iter()
                    .map(|state| {
                        quote! {
                          #machine_name::#state(ref v) => Some(v.#ident()),
                        }
                    })
                    .collect::<Vec<_>>();
                quote! {
                  pub fn #ident(&self) -> Option<&#ty> {
                    match self {
                      #(#variants)*
                      _ => None,
                    }
                  }
                }
            }
            MethodType::Set(ident, ty) => {
                let mut_ident =
                    Ident::new(&format!("{}_mut", &ident.to_string()), Span::call_site());

                let variants = method
                    .states
                    .iter()
                    .map(|state| {
                        quote! {
                          #machine_name::#state(ref mut v) => Some(v.#mut_ident()),
                        }
                    })
                    .collect::<Vec<_>>();
                quote! {
                  pub fn #mut_ident(&mut self) -> Option<&mut #ty> {
                    match self {
                      #(#variants)*
                      _ => None,
                    }
                  }
                }
            }
            MethodType::Fn(m) => {
                let ident = &m.ident;
                let args = m
                    .decl
                    .inputs
                    .iter()
                    .filter(|arg| match arg {
                        FnArg::Captured(_) => true,
                        _ => false,
                    })
                    .map(|arg| {
                        if let FnArg::Captured(a) = arg {
                            &a.pat
                        } else {
                            panic!();
                        }
                    })
                    .collect::<Vec<_>>();

                let variants = method
                    .states
                    .iter()
                    .map(|state| {
                        let a = args.clone();
                        if method.default.is_default() {
                            quote! {
                              #machine_name::#state(ref v) => v.#ident( #(#a),* ),
                            }
                        } else {
                            quote! {
                              #machine_name::#state(ref v) => Some(v.#ident( #(#a),* )),
                            }
                        }
                    })
                    .collect::<Vec<_>>();

                let inputs = &m.decl.inputs;
                let output = match &m.decl.output {
                    ReturnType::Default => quote! {},
                    ReturnType::Type(arrow, ty) => {
                        if method.default.is_default() {
                            quote! {
                              #arrow #ty
                            }
                        } else {
                            quote! {
                              #arrow Option<#ty>
                            }
                        }
                    }
                };

                match method.default {
                    DefaultValue::None => {
                        quote! {
                          pub fn #ident(#inputs) #output {
                            match self {
                              #(#variants)*
                              _ => None,
                            }
                          }
                        }
                    }
                    DefaultValue::Default => {
                        quote! {
                          pub fn #ident(#inputs) #output {
                            match self {
                              #(#variants)*
                              _ => std::default::Default::default(),
                            }
                          }
                        }
                    }
                    DefaultValue::Val(ref expr) => {
                        quote! {
                          pub fn #ident(#inputs) #output {
                            match self {
                              #(#variants)*
                              _ => #expr,
                            }
                          }
                        }
                    }
                }
            }
        })
        .collect::<Vec<_>>();

    let toks = quote! {
      impl #machine_name {
        #(#wrapper_methods)*
      }
    };

    stream.extend(proc_macro::TokenStream::from(toks));

    let file_name = format!("target/machine/{}.rs", machine_name.to_string().to_lowercase());
    let _ = create_dir("target/machine");
    OpenOptions::new()
        .create(true)
        .write(true)
        .open(&file_name)
        .and_then(|mut file| {
            file.seek(std::io::SeekFrom::End(0))?;
            file.write_all(stream.to_string().as_bytes())?;
            file.flush()
        })
        .expect("error writing methods");

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
    pub default: DefaultValue,
}

#[derive(Debug)]
enum MethodType {
    Get(Ident, Type),
    Set(Ident, Type),
    Fn(MethodSig),
}

#[derive(Debug)]
enum DefaultValue {
    None,
    Default,
    Val(Expr),
}

impl DefaultValue {
    pub fn is_default(&self) -> bool {
        match self {
            DefaultValue::None => false,
            _ => true,
        }
    }
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
            machine_name,
            methods,
        })
    }
}
struct ParenVal {
    expr: Expr,
}

impl Parse for ParenVal {
    fn parse(input: ParseStream) -> Result<Self> {
        let stream;
        parenthesized!(stream in input);
        let expr: Expr = stream.parse()?;
        Ok(ParenVal { expr })
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
        let default_token: Option<Token![default]> = input.parse()?;
        let default = if default_token.is_some() {
            match input.parse::<ParenVal>() {
                Ok(content) => DefaultValue::Val(content.expr),
                Err(_) => DefaultValue::Default,
            }
        } else {
            DefaultValue::None
        };

        let method_type = match parse_method_sig(input) {
            Ok(f) => MethodType::Fn(f),
            Err(_) => {
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

        Ok(Method {
            states,
            method_type,
            default,
        })
    }
}

fn parse_method_sig(input: ParseStream) -> Result<MethodSig> {
    //let vis: Visibility = input.parse()?;
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
        constness,
        unsafety,
        asyncness,
        abi,
        ident,
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
        },
    })
}

fn type_to_snake(t: &Type) -> String {
  match t {
    Type::Path(ref p) => {
      match p.path.segments.last() {
        Some(Pair::End(segment)) => {
          segment.ident.to_string().to_snake()
        },
        _ => panic!("expected a path segment"),
      }
    },
    t => panic!("expected a Type::Path, got {:?}", t),
  }
}

fn type_last_ident(t: &Type) -> &Ident {
  match t {
    Type::Path(ref p) => {
      match p.path.segments.last() {
        Some(Pair::End(segment)) => {
          &segment.ident
        },
        _ => panic!("expected a path segment"),
      }
    },
    t => panic!("expected a Type::Path, got {:?}", t),
  }
}

fn type_args(t: &Type) -> HashSet<GenericArgument> {
  match t {
    Type::Path(ref p) => {
      match p.path.segments.last() {
        Some(Pair::End(segment)) => {
          match &segment.arguments {
            PathArguments::AngleBracketed(a) => {
              a.args.iter().cloned().collect()
            },
            PathArguments::None => HashSet::new(),
            a => panic!("expected angle bracketed arguments, got {:?}", a),
          }
        },
        _ => panic!("expected a path segment"),
      }
    },
    t => panic!("expected a Type::Path, got {:?}", t),
  }
}

// lifetimes must appear before other type arguments
fn reorder_type_arguments(mut t: HashSet<GenericArgument>) -> Vec<GenericArgument> {
  let mut lifetimes = Vec::new();
  let mut others = Vec::new();

  for arg in t.drain() {
    if let GenericArgument::Lifetime(_) = arg {
      lifetimes.push(arg);
    } else {
      others.push(arg);
    }
  }

  lifetimes.extend(others.drain(..));
  lifetimes
}
