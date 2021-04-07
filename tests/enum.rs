extern crate minecraft_packet_derive;
use minecraft_packet_derive::*;

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

#[minecraft_enum(u8)]
pub enum TestEnum {
    West = 1,
    North,
    East = 54,
    South,
}