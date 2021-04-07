extern crate proc_macro;
use proc_macro::{TokenStream, Span};
use quote::{quote, format_ident};
use syn::{parse_macro_input, Data, DeriveInput, Fields, Expr, Lit, LitInt};

#[proc_macro_derive(MinecraftPacket)]
pub fn minecraft_packet_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let (lifetime_impl, lifetime_struct, lifetime) = match input.generics.lifetimes().collect::<Vec<&_>>() {
        lifetimes if lifetimes.is_empty() => (quote!{<'a>}, None, None),
        lifetimes if lifetimes.len() == 1 => {
            let lifetime = lifetimes[0].lifetime.clone();
            (quote! {<#lifetime>}, Some(quote! {<#lifetime>}), Some(quote! {#lifetime}))
        },
        _ => return quote!(compile_error!("Too many lifetimes");).into()
    };
 
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
                impl#lifetime_impl MinecraftPacket#lifetime_impl for #name#lifetime_struct {
                    fn serialize(self) -> Result<Vec<u8>, &'static str> {
                        let mut output = Vec::new();
                        #(self.#fields.append_minecraft_packet_part(&mut output)?;)*
                        Ok(output)
                    }

                    fn deserialize(mut input: &#lifetime mut [u8]) -> Result<Self, &'static str> {
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

#[proc_macro_attribute]
pub fn minecraft_enum(attr: TokenStream, input: TokenStream) -> TokenStream {
    let argument_type = attr.to_string();
    let representation_type = match argument_type.as_str() == "VarInt" {
        true => "i32".to_string(),
        false => argument_type.clone(),
    };
    let representation_ident = format_ident!("{}", representation_type);
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident.clone();

    let data = match input.data.clone() {
        Data::Enum(data) => data,
        _ => return quote!(compile_error!("Unsupported data structure");).into()
    };

    let unmatched_message = format!("The {} ID is outside the definition range.", name.to_string());

    let mut variant_name = Vec::new();
    let mut variant_value = Vec::new();
    let mut last_discriminant = 0;
    for variant in data.variants {
        let discriminant = if let Some((_, Expr::Lit(d))) = variant.discriminant {
            println!("{:?}", d);
            if let Lit::Int(d) = d.lit {
                d.base10_parse::<i64>().unwrap()
            } else {
                last_discriminant + 1
            }
        } else {
            last_discriminant + 1
        };
        last_discriminant = discriminant;
        let discriminant = Lit::Int(LitInt::new(&format!("{}{}", discriminant, representation_type), Span::call_site().into()));
        variant_name.push(variant.ident);
        variant_value.push(discriminant);
    };

    let append_implementation = match argument_type.as_str() {
        "u8" => {
            quote! {
                output.push(self as u8);
                Ok(())
            }
        }
        "VarInt" => {
            quote! {
                VarInt(self as i32).append_minecraft_packet_part(output)
            }
        }
        _ => quote! {
            (self as #representation_type).append_minecraft_packet_part(output)
        },
    };

    let build_implementation = match argument_type.as_str() {
        "VarInt" =>quote! {
            let (id, input) = VarInt::build_from_minecraft_packet(input)?;
            let value = match id.0 {
                #(#variant_value => #name::#variant_name,)*
                _ => return Err(#unmatched_message),
            };
            Ok((value, input))
        },
        _ => quote! {
            let (id, input) = #representation_ident::build_from_minecraft_packet(input)?;
            let value = match id {
                #(#variant_value => #name::#variant_name,)*
                _ => return Err(#unmatched_message),
            };
            Ok((value, input))
        },
    };

    {quote! {
        #[repr(#representation_ident)]
        #input

        impl<'a> MinecraftPacketPart<'a> for #name {
            fn append_minecraft_packet_part(self, output: &mut Vec<u8>) -> Result<(), &'static str> {
                #append_implementation
            }

            fn build_from_minecraft_packet(input: &'a mut [u8]) -> Result<(Self, &'a mut [u8]), &'static str> {
                #build_implementation
            }
        }
    }}.into()
}
