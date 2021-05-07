use crate::packet::{BytePacketBuffer, Packet};
use std::{fs::File, io::Read};
mod packet;

type Error = Box<dyn std::error::Error>;
type Result<T> = std::result::Result<T, Error>;

fn main() -> Result<()> {
    println!("Hello, world!");
    let mut f = File::open("response_packet.txt")?;
    let mut buffer = BytePacketBuffer::new();
    println!("{:#?}",f);
    f.read(&mut buffer.buffer)?;

    let packet = Packet::from_buffer(&mut buffer)?;
    println!("{:#?}",packet.header);
    for q in packet.question {
        println!("{:#?}", q);
    }
    for q in packet.answer {
        println!("{:#?}", q);
    }
    for q in packet.authorities {
        println!("{:#?}", q);
    }
    for q in packet.resources {
        println!("{:#?}", q);
    }
    Ok(())
}
