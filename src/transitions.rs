use std::collections::HashMap;
use std::fs::File;
use std::io::Write;

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
    pub fn render(&self) {
        let file_name = format!(
            "target/{}.dot",
            self.machine_name.to_string().to_lowercase()
        );
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
                &format!("{} -> {} [ label = \"{}\" ];\n", edge.0, edge.2, edge.1).as_bytes(),
            )
            .expect("error writing to dot file");
        }

        file.write_all(&b"}"[..])
            .expect("error writing to dot file");
        file.flush().expect("error flushhing dot file");
    }

    pub fn generate(&self) -> (&Ident, syn::export::TokenStream) {
        //println!("\ninput: {:?}", input);
        let mut stream = proc_macro::TokenStream::new();

        self.render();

        let machine_name = &self.machine_name;

        let mut messages = HashMap::new();
        for t in self.transitions.iter() {
            let entry = messages.entry(&t.message).or_insert(Vec::new());
            entry.push((&t.start, &t.end));
        }

        // create an enum from the messages
        let message_enum_ident = Ident::new(
            &format!("{}Messages", &machine_name.to_string()),
            Span::call_site(),
        );
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

        let functions = messages
            .iter()
            .map(|(msg, moves)| {
                let fn_ident = Ident::new(
                    &format!("on_{}", &msg.to_string().to_snake()),
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

                quote! {
                  pub fn #fn_ident(self, input: #msg) -> #machine_name {
                    match self {
                    #(#mv)*
                      _ => #machine_name::Error,
                    }
                  }
                }
            })
            .collect::<Vec<_>>();

        let toks = quote! {
          impl #machine_name {
            #(#functions)*
          }
        };

        stream.extend(proc_macro::TokenStream::from(toks));

        (machine_name, stream)
    }
}
