extern crate minecraft_packet_derive;
use minecraft_packet_derive::*;

#[derive(Debug, MinecraftPacket)]
pub struct Test {
    data: u8,
    other: String,
}
pub trait MinecraftPacket: Sized {
    fn serialize(self) -> Result<Vec<u8>, &'static str>;
    fn deserialize(input: Vec<u8>) -> Result<Self, &'static str>;
}

pub trait MinecraftPacketPart: Sized {
    fn append_minecraft_packet_part(self, output: &mut Vec<u8>) -> Result<(), &'static str>;
    fn build_from_minecraft_packet(input: &mut [u8]) -> Result<(Self, &mut [u8]), &'static str>;
}

impl MinecraftPacketPart for u8 {
    fn append_minecraft_packet_part(self, output: &mut Vec<u8>) -> Result<(), &'static str> {
        output.push(self);
        Ok(())
    }

    fn build_from_minecraft_packet(input: &mut [u8]) -> Result<(Self, &mut [u8]), &'static str> {
        let (value, input) = input.split_first_mut().unwrap();
        Ok((*value, input))
    }
}

impl MinecraftPacketPart for String {
    fn append_minecraft_packet_part(self, output: &mut Vec<u8>) -> Result<(), &'static str> {
        output.push(self.len() as u8);
        output.extend_from_slice(self.as_bytes());
        Ok(())
    }

    fn build_from_minecraft_packet(input: &mut [u8]) -> Result<(Self, &mut [u8]), &'static str> {
        let (len, input) = input.split_first_mut().unwrap();
        let (slice, input) = input.split_at_mut(*len as usize);
        Ok((String::from_utf8(slice.to_vec()).unwrap(), input))
    }
}

#[test]
fn main() {
    let data = Test {
        data: 5,
        other: String::from("heyyy"),
    };
    let serialized = data.serialize().unwrap();
    let deserialized = Test::deserialize(serialized.clone()).unwrap();
    dbg!(serialized, deserialized);
}
