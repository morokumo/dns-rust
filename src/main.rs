use std::net::UdpSocket;

use crate::{buffer::BytePacketBuffer, enums::QueryType, packet::Packet, question::Question};
mod buffer;
mod enums;
mod header;
mod packet;
mod question;

type Error = Box<dyn std::error::Error>;
type Result<T> = std::result::Result<T, Error>;

const PACKET_BUFFER_SIZE: usize = 512;

fn main() -> Result<()> {
    let socket = UdpSocket::bind(("0.0.0.0", 8053))?;
    loop {
        match handle_query(&socket) {
            Ok(_) => {}
            Err(e) => eprintln!("ERROR! :{}", e),
        }
    }
}

fn lookup(qname: &str, qtype: QueryType) -> Result<Packet> {
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
    socket.send_to(&req_buffer.buffer[0..req_buffer.pos()], server)?;

    let mut res_buffer = BytePacketBuffer::new();
    socket.recv_from(&mut res_buffer.buffer)?;

    Packet::from_buffer(&mut res_buffer)
}

fn handle_query(socket: &UdpSocket) -> Result<()> {
    let mut req_buffer = BytePacketBuffer::new();
    let (_, src) = socket.recv_from(&mut req_buffer.buffer)?;

    let mut request = Packet::from_buffer(&mut req_buffer)?;

    let mut packet = Packet::new();
    packet.header.id = request.header.id;
    packet.header.recursion_desired = true;
    packet.header.recursion_available = true;
    packet.header.responce = true;

    if let Some(question) = request.questions.pop() {
        println!("rcvd query : {:?}", question);

        if let Ok(result) = lookup(&question.name, question.qtype) {
            packet.questions.push(question);
            packet.header.responce_code = result.header.responce_code;

            for q in result.answers {
                println!("{:#?}", q);
            }
            for q in result.authorities {
                println!("{:#?}", q);
            }
            for q in result.resources {
                println!("{:#?}", q);
            }
        } else {
            packet.header.responce_code = enums::ResultCode::SERVFAIL;
        }
    } else {
        packet.header.responce_code = enums::ResultCode::FORMERR;
    }

    let mut res_buffer = BytePacketBuffer::new();
    packet.write(&mut res_buffer)?;

    let len = res_buffer.pos();
    let data = res_buffer.get_range(0, len)?;

    socket.send_to(data, src)?;

    Ok(())
}
