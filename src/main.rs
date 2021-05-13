use std::net::UdpSocket;

use crate::handler::handle_query;
mod buffer;
mod enums;
mod handler;
mod header;
mod packet;
mod question;
mod record;

const PACKET_BUFFER_SIZE: usize = 512;

type Error = Box<dyn std::error::Error>;
type Result<T> = std::result::Result<T, Error>;

fn main() -> Result<()> {
    let socket = UdpSocket::bind(("0.0.0.0", 8053))?;
    loop {
        match handle_query(&socket) {
            Ok(_) => {}
            Err(e) => eprintln!("ERROR! :{}", e),
        }
    }
}
