extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields};

fn minecraft_packet_addition(_attr: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = input.ident;

    let data = match input.data {
        Data::Struct(data) => data,
        _ => return quote!(compile_error!("Unsupported data structure");).into()
    };
    
    match data.fields {
        Fields::Named(fields) => {
            let fields = fields.named.into_iter().map(|field| field.ident.unwrap());
            quote! {
                impl MinecraftPacket for #name {
                    fn serialize(self) -> Result<Vec<u8>, &'static str> {
                        let mut output = Vec::new();
                        #(self.#fields.append_minecraft_packet_part(&mut output)?;)*
                        Ok(output)
                    }
                }
            }
        }
        Fields::Unnamed(_) => todo!("unnamed fields"),
        Fields::Unit => panic!("how did you put a variant in a struct??"),
    }.into()
}


#[proc_macro_attribute]
pub fn minecraft_packet(attr: TokenStream, mut input: TokenStream) -> TokenStream {
    input.extend(minecraft_packet_addition(attr, input.clone()));
    input
}
