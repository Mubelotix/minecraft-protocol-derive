extern crate minecraft_packet_derive;
use minecraft_packet_derive::*;

#[derive(Debug, MinecraftPacket, PartialEq, Clone)]
pub struct Test<'a> {
    data: u8,
    other: &'a str,
}
pub trait MinecraftPacket<'a>: Sized {
    fn serialize(self) -> Result<Vec<u8>, &'static str>;
    fn deserialize(input: &'a mut [u8]) -> Result<Self, &'static str>;
}

pub trait MinecraftPacketPart<'a>: Sized {
    fn append_minecraft_packet_part(self, output: &mut Vec<u8>) -> Result<(), &'static str>;
    fn build_from_minecraft_packet(input: &'a mut [u8]) -> Result<(Self, &'a mut [u8]), &'static str>;
}

impl<'a> MinecraftPacketPart<'a> for u8 {
    fn append_minecraft_packet_part(self, output: &mut Vec<u8>) -> Result<(), &'static str> {
        output.push(self);
        Ok(())
    }

    fn build_from_minecraft_packet(input: &mut [u8]) -> Result<(Self, &mut [u8]), &'static str> {
        let (value, input) = input.split_first_mut().unwrap();
        Ok((*value, input))
    }
}

impl<'a> MinecraftPacketPart<'a> for &'a str {
    fn append_minecraft_packet_part(self, output: &mut Vec<u8>) -> Result<(), &'static str> {
        output.push(self.len() as u8);
        output.extend_from_slice(self.as_bytes());
        Ok(())
    }

    fn build_from_minecraft_packet(input: &'a mut [u8]) -> Result<(Self, &'a mut [u8]), &'static str> {
        let (len, input) = input.split_first_mut().unwrap();
        let (slice, input) = input.split_at_mut(*len as usize);
        Ok((std::str::from_utf8(slice).unwrap(), input))
    }
}

#[test]
fn main() {
    let data = Test {
        data: 5,
        other: "sfd"
    };
    let mut serialized = data.clone().serialize().unwrap();
    let deserialized = Test::deserialize(&mut serialized).unwrap();
    assert_eq!(data, deserialized);
}
