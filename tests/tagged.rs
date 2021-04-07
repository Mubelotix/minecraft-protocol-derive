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


#[derive(MinecraftStructuredEnum)]
#[discriminant(u8)]
pub enum TestEnum<'a> {
    Teacher {student_count: u8, grade_average: u8},
    #[value = 5]
    Farmer {root_meters_count: u8, name: &'a str}
}
