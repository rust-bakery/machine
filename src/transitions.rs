use std::collections::HashMap;

use case::CaseExt;
use syn::export::Span;
use syn::parse::{Parse, ParseStream, Result};
use syn::Ident;

#[derive(Debug)]
pub struct Transitions {
    pub machine_name: Ident,
    pub transitions: Vec<Transition>,
}

#[derive(Debug)]
pub struct Transition {
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

        Ok(Transition {
            start,
            message,
            end,
        })
    }
}

impl Transitions {
    pub fn render_dot(&self) -> String {
        let mut string = format!("digraph {} {{\n", self.machine_name.to_string());

        let mut edges = Vec::new();
        for transition in self.transitions.iter() {
            for state in transition.end.iter() {
                edges.push((&transition.start, &transition.message, state));
            }
        }

        for edge in edges.iter() {
            string.push_str(&format!(
                "    {} -> {} [ label = \"{}\" ];\n",
                edge.0, edge.2, edge.1
            ))
        }

        string.push_str("}");

        string
    }

    pub fn generate(&self) -> (&Ident, syn::export::TokenStream) {
        let mut stream = proc_macro::TokenStream::new();

        let machine_name = &self.machine_name;

        let mut messages = HashMap::new();
        for t in self.transitions.iter() {
            let entry = messages.entry(&t.message).or_insert(Vec::new());
            entry.push((&t.start, &t.end));
        }

        stream.extend(self.generate_messages_enum(&messages));
        stream.extend(self.generate_impl(&messages));

        (machine_name, stream)
    }

    fn generate_messages_enum(
        &self,
        messages: &HashMap<&syn::Ident, Vec<(&syn::Ident, &Vec<syn::Ident>)>>,
    ) -> syn::export::TokenStream {
        let machine_name = &self.machine_name;

        // create an enum from the messages
        let message_enum_ident = Ident::new(
            &format!("{}Messages", &machine_name.to_string()),
            Span::call_site(),
        );
        let variants_names = &messages.keys().collect::<Vec<_>>();
        let structs_names = variants_names.clone();

        let tokens = quote! {
          #[derive(Clone,Debug,PartialEq)]
          pub enum #message_enum_ident {
            #(#variants_names(#structs_names)),*
          }
        };

        proc_macro::TokenStream::from(tokens)
    }

    fn generate_fn(
        &self,
        message: &syn::Ident,
        moves: &[(&syn::Ident, &Vec<syn::Ident>)],
    ) -> syn::export::TokenStream2 {
        let machine_name = &self.machine_name;

        let fn_ident = Ident::new(
            &format!("on_{}", &message.to_string().to_snake()),
            Span::call_site(),
        );

        let mv: Vec<_> = moves.iter().map(|(start, end)| {
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
        }).collect();

        quote! {
            pub fn #fn_ident(self, input: #message) -> #machine_name {
                match self {
                #(#mv)*
                    _ => #machine_name::Error,
                }
            }
        }
    }

    fn generate_impl(
        &self,
        messages: &HashMap<&syn::Ident, Vec<(&syn::Ident, &Vec<syn::Ident>)>>,
    ) -> syn::export::TokenStream {
        let machine_name = &self.machine_name;

        let functions = messages
            .iter()
            .map(|(message, moves)| self.generate_fn(message, moves.as_slice()))
            .collect::<Vec<_>>();

        let tokens = quote! {
          impl #machine_name {
            #(#functions)*
          }
        };

        proc_macro::TokenStream::from(tokens)
    }
}
