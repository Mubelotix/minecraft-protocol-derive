extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields};

#[proc_macro_derive(MinecraftPacket)]
pub fn minecraft_packet_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = input.ident;

    let data = match input.data {
        Data::Struct(data) => data,
        _ => return quote!(compile_error!("Unsupported data structure");).into()
    };
    
    match data.fields {
        Fields::Named(fields) => {
            let fields = fields.named.into_iter().map(|field| field.ident.unwrap());
            let fields2 = fields.clone();
            let fields3 = fields.clone();

            quote! {
                #[automatically_derived]
                impl MinecraftPacket for #name {
                    fn serialize(self) -> Result<Vec<u8>, &'static str> {
                        let mut output = Vec::new();
                        #(self.#fields.append_minecraft_packet_part(&mut output)?;)*
                        Ok(output)
                    }

                    fn deserialize(mut input: &mut [u8]) -> Result<Self, &'static str> {
                        #(let (#fields2, input) = MinecraftPacketPart::build_from_minecraft_packet(input)?;)*
                        if !input.is_empty() {
                            return Err("A few bytes are remaining after deserialization.");
                        }
                        Ok(#name {
                            #(#fields3,)*
                        })
                    }
                }
            }
        }
        Fields::Unnamed(_) => todo!("unnamed fields"),
        Fields::Unit => panic!("how did you put a variant in a struct??"),
    }.into()
}
