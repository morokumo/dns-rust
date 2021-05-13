use std::{fs::File, io::Read, net::UdpSocket};

use crate::{buffer::BytePacketBuffer, enums::QueryType, packet::Packet, question::Question};
mod packet;
mod buffer;
mod header;
mod enums;
mod question;

type Error = Box<dyn std::error::Error>;
type Result<T> = std::result::Result<T, Error>;

const PACKET_BUFFER_SIZE: usize = 512;

fn main() -> Result<()> {
    println!("Hello, world!");
    let mut f = File::open("response_packet.txt")?;
    let mut buffer = BytePacketBuffer::new();
    println!("{:#?}", f);
    f.read(&mut buffer.buffer)?;

    let packet = Packet::from_buffer(&mut buffer)?;
    println!("{:#?}", packet.header);
    for q in packet.questions {
        println!("{:#?}", q);
    }
    for q in packet.answers {
        println!("{:#?}", q);
    }
    for q in packet.authorities {
        println!("{:#?}", q);
    }
    for q in packet.resources {
        println!("{:#?}", q);
    }

    println!("<#>-------------------");

    let qname = "yahoo.com";
    let qtype = QueryType::MX;

    let server = ("8.8.8.8", 53);
    let socket = UdpSocket::bind(("0.0.0.0", 43210))?;

    let mut packet = Packet::new();

    packet.header.id = 6666;
    packet.header.questions = 1;
    packet.header.recursion_desired = true;
    packet
        .questions
        .push(Question::new(qname.to_string(), qtype));

    let mut req_buffer = BytePacketBuffer::new();
    packet.write(&mut req_buffer)?;
    socket.send_to(&req_buffer.buffer[0..req_buffer.pos], server)?;

    let mut res_buffer = BytePacketBuffer::new();
    socket.recv_from(&mut res_buffer.buffer)?;

    let res_packet = Packet::from_buffer(&mut res_buffer)?;
    println!("{:#?}", res_packet.header);

    for q in res_packet.questions {
        println!("{:#?}", q);
    }
    for q in res_packet.answers {
        println!("{:#?}", q);
    }
    for q in res_packet.authorities {
        println!("{:#?}", q);
    }
    for q in res_packet.resources {
        println!("{:#?}", q);
    }

    Ok(())
}
