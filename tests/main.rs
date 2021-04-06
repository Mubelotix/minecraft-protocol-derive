extern crate minecraft_packet_derive;
use minecraft_packet_derive::minecraft_packet;

#[minecraft_packet]
pub struct Test {
    data: u8,
    other: String,
}
pub trait MinecraftPacket {
    fn serialize(self) -> Result<Vec<u8>, &'static str>;
}

pub trait MinecraftPacketPart {
    fn append_minecraft_packet_part(self, output: &mut Vec<u8>) -> Result<(), &'static str>;
}

impl MinecraftPacketPart for u8 {
    fn append_minecraft_packet_part(self, output: &mut Vec<u8>) -> Result<(), &'static str> {
        todo!()
    }
}

impl MinecraftPacketPart for String {
    fn append_minecraft_packet_part(self, output: &mut Vec<u8>) -> Result<(), &'static str> {
        todo!()
    }
}

#[test]
fn main() {
    let data = Test{data: 5, other: String::from("heyyy")};
    println!("{:?}", data.serialize())
}