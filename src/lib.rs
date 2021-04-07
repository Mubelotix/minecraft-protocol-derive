extern crate proc_macro;
use proc_macro::{Span, TokenStream};
use quote::{format_ident, quote};
use syn::{parse_macro_input, Data, DeriveInput, Expr, Fields, Lit, LitInt, punctuated::Punctuated};

#[proc_macro_derive(MinecraftPacket)]
pub fn minecraft_packet_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let (lifetime_impl, lifetime_struct, lifetime) =
        match input.generics.lifetimes().collect::<Vec<&_>>() {
            lifetimes if lifetimes.is_empty() => (quote! {<'a>}, None, None),
            lifetimes if lifetimes.len() == 1 => {
                let lifetime = lifetimes[0].lifetime.clone();
                (
                    quote! {<#lifetime>},
                    Some(quote! {<#lifetime>}),
                    Some(quote! {#lifetime}),
                )
            }
            _ => return quote!(compile_error!("Too many lifetimes");).into(),
        };

    let name = input.ident;

    let data = match input.data {
        Data::Struct(data) => data,
        _ => return quote!(compile_error!("Unsupported data structure");).into(),
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
    // Collect data
    let argument_type = attr.to_string();
    let representation_type = match argument_type.as_str() {
        "VarInt" => "i32".to_string(),
        "i8" | "u8" | "i16" | "u16" | "i32" | "u32" | "i64" => argument_type.clone(),
        _ => return quote!(compile_error!("Unsupported tag type");).into(),
    };
    let representation_ident = format_ident!("{}", representation_type);
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident.clone();
    let data = match input.data.clone() {
        Data::Enum(data) => data,
        _ => return quote!(compile_error!("Unsupported data structure");).into(),
    };
    let unmatched_message = format!(
        "The {} ID is outside the definition range.",
        name.to_string()
    );

    // Analyse enum variants
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
        let discriminant = Lit::Int(LitInt::new(
            &format!("{}{}", discriminant, representation_type),
            Span::call_site().into(),
        ));
        variant_name.push(variant.ident);
        variant_value.push(discriminant);
    }

    // Construct the append_minecraft_packet_part method
    let append_implementation = match argument_type.as_str() {
        "u8" => quote! {
            output.push(self as u8);
            Ok(())
        },
        "VarInt" => quote! {
            VarInt(self as i32).append_minecraft_packet_part(output)
        },
        _ => quote! {
            (self as #representation_type).append_minecraft_packet_part(output)
        },
    };

    // Construct the build_from_minecraft_packet method
    let build_implementation = match argument_type.as_str() {
        "VarInt" => quote! {
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

    // Derive MinecraftPacketPart
    {quote! {
        #[repr(#representation_ident)]
        #input

        #[automatically_derived]
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

#[proc_macro_derive(MinecraftStructuredEnum, attributes(discriminant, value))]
pub fn minecraft_tagged(input: TokenStream) -> TokenStream {
    // Collect the data
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident.clone();
    let mut tag_type_string = "u8".to_string();
    for attr in input.attrs {
        if let Some(path) = attr.path.segments.first() {
            match path.ident.to_string().as_str() {
                "discriminant" => {
                    tag_type_string = attr.tokens.to_string();
                    if tag_type_string.starts_with('(') && tag_type_string.ends_with(')') {
                        tag_type_string.remove(tag_type_string.len() - 1);
                        tag_type_string.remove(0);
                    }
                },
                "value" => return quote!(compile_error!("Not the right place for value attribute");).into(),
                _ => (),
            }
        }
    }
    let tag_type_ident = format_ident!("{}", tag_type_string);
    let unmatched_message = format!(
        "The {} ID is outside the definition range.",
        name.to_string()
    );
    let variants = match input.data {
        Data::Enum(variants) => variants.variants,
        _ => return quote!(compile_error!("Unsupported data structure");).into(),
    };

    // Collect lifetimes
    let (lifetime_impl, lifetime_struct, lifetime) =
        match input.generics.lifetimes().collect::<Vec<&_>>() {
            lifetimes if lifetimes.is_empty() => (quote! {<'a>}, None, None),
            lifetimes if lifetimes.len() == 1 => {
                let lifetime = lifetimes[0].lifetime.clone();
                (
                    quote! {<#lifetime>},
                    Some(quote! {<#lifetime>}),
                    Some(quote! {#lifetime}),
                )
            }
            _ => return quote!(compile_error!("Too many lifetimes");).into(),
        };


    // Process variants one by one
    let mut serialization_arms = Vec::new();
    let mut deserialization_arms = Vec::new();
    let mut last_discriminant = 0;
    for variant in variants {
        // Collect variant data
        let mut discriminant = last_discriminant + 1;
        for attr in variant.attrs {
            if let Some(path) = attr.path.segments.first() {
                match path.ident.to_string().as_str() {
                    "discriminant" => return quote!(compile_error!("Not the right place for discriminant attribute");).into(),
                    "value" => {
                        let mut discriminant_string = attr.tokens.to_string();
                        if discriminant_string.starts_with(' ') {
                            discriminant_string.remove(0);
                        }
                        if discriminant_string.starts_with('=') {
                            discriminant_string.remove(0);
                        } else {
                            return quote!(compile_error!("Invalid value attribute");).into();
                        }
                        if discriminant_string.starts_with(' ') {
                            discriminant_string.remove(0);
                        }
                        println!("{}", discriminant_string);
                        discriminant = discriminant_string.parse().unwrap();
                    },
                    _ => (),
                }
            }
        }
        let discriminant_lit = Lit::Int(LitInt::new(
            &format!("{}{}", discriminant, tag_type_string),
            Span::call_site().into(),
        ));
        last_discriminant = discriminant;
        let variant_name = variant.ident;
        let fields = variant.fields;
        let fields = match fields {
            Fields::Named(fields) => fields.named,
            Fields::Unit => Punctuated::new(),
            _ => return quote!(compile_error!("All fields must be named");).into(),
        };

        // Build a serialization arm
        let field_names = fields.iter().map(|field| field.ident.as_ref().unwrap());
        let field_names2 = fields.iter().map(|field| field.ident.as_ref().unwrap());
        let serialization_arm = quote! {
            #name::#variant_name{#(#field_names2, )*} => {
                #discriminant_lit.append_minecraft_packet_part(output)?;
                #(#field_names.append_minecraft_packet_part(output)?;)*
                Ok(())
            },
        };
        serialization_arms.push(serialization_arm);

        // Build a deserialization arm
        let field_names = fields.iter().map(|field| field.ident.as_ref().unwrap());
        let field_names2 = fields.iter().map(|field| field.ident.as_ref().unwrap());
        let field_types = fields.iter().map(|field| &field.ty);
        let deserialization_arm = quote! {
            #discriminant_lit => {
                #(let (#field_names, input) = <#field_types>::build_from_minecraft_packet(input)?;)*
                Ok((#name::#variant_name {
                    #(#field_names2, )*
                }, input))
            },
        };
        deserialization_arms.push(deserialization_arm);
    }

    // Gather serialization arms
    let serialization_implementation = quote! {
        match self {
            #(#serialization_arms)*
        }
    };

    // Gather deserialization arms
    let deserialization_implementation = quote! {
        let (id, input) = #tag_type_ident::build_from_minecraft_packet(input)?;
        match id {
            #(#deserialization_arms)*
            _ => Err(#unmatched_message),
        }
    };

    // Derive MinecraftPacketPart
    {quote! {
        #[automatically_derived]
        impl#lifetime_impl MinecraftPacketPart#lifetime_impl for #name#lifetime_struct {
            fn append_minecraft_packet_part(self, output: &mut Vec<u8>) -> Result<(), &'static str> {
                #serialization_implementation
            }

            fn build_from_minecraft_packet(input: &#lifetime mut [u8]) -> Result<(Self, &#lifetime mut [u8]), &'static str> {
                #deserialization_implementation
            }
        }
    }}.into()
}
