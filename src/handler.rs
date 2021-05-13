use std::net::{Ipv4Addr, UdpSocket};

use crate::{
    buffer::BytePacketBuffer,
    enums::{QueryType, ResultCode},
    packet::Packet,
    question::Question,
};

type Error = Box<dyn std::error::Error>;
type Result<T> = std::result::Result<T, Error>;

pub fn handle_query(socket: &UdpSocket) -> Result<()> {
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

        if let Ok(result) = recursice_lookup(&question.name, question.qtype) {
            packet.questions.push(question.clone());
            packet.header.responce_code = result.header.responce_code;

            for q in result.answers {
                println!("Answer : {:?}", q);
                packet.answers.push(q);
            }
            for q in result.authorities {
                println!("Authority : {:?}", q);
                packet.authorities.push(q);
            }
            for q in result.resources {
                println!("Resource : {:?}", q);
                packet.resources.push(q);
            }
        } else {
            packet.header.responce_code = ResultCode::SERVFAIL;
        }
    } else {
        packet.header.responce_code = ResultCode::FORMERR;
    }

    let mut res_buffer = BytePacketBuffer::new();
    packet.write(&mut res_buffer)?;

    let len = res_buffer.pos();
    let data = res_buffer.get_range(0, len)?;

    socket.send_to(data, src)?;
    println!("{:#?}", packet);
    Ok(())
}

fn lookup(qname: &str, qtype: QueryType, server: (Ipv4Addr, u16)) -> Result<Packet> {
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

fn recursice_lookup(qname: &str, qtype: QueryType) -> Result<Packet> {
    let mut ns = "198.41.0.4".parse::<Ipv4Addr>()?;
    loop {
        println!("attemptiong lookup of {:?} {} with ns {}", qtype, qname, ns);
        let ns_copy = ns;
        let server = (ns_copy, 53);
        let response = lookup(qname, qtype, server)?;
        if !response.answers.is_empty() && response.header.responce_code == ResultCode::NOERROR {
            return Ok(response);
        }

        if response.header.responce_code == ResultCode::NXDOMAIN {
            return Ok(response);
        }
        if let Some(new_ns) = response.get_resolved_ns(qname) {
            ns = new_ns;
            continue;
        }
        let new_ns_name = match response.get_unresolved_ns(qname) {
            Some(x) => x,
            None => return Ok(response),
        };

        let recursive_response = recursice_lookup(&new_ns_name, QueryType::A)?;

        if let Some(new_ns) = recursive_response.get_random_a() {
            ns = new_ns;
        } else {
            return Ok(response);
        }
    }
}
