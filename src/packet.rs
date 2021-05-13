use std::net::Ipv4Addr;

use crate::{buffer::BytePacketBuffer, header::Header, question::Question};
use crate::{enums::QueryType, record::DnsRecord};

type Error = Box<dyn std::error::Error>;
type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub struct Packet {
    pub header: Header,
    pub questions: Vec<Question>,
    pub answers: Vec<DnsRecord>,
    pub authorities: Vec<DnsRecord>,
    pub resources: Vec<DnsRecord>,
}

impl Packet {
    pub fn new() -> Self {
        Self {
            header: Header::new(),
            questions: Vec::new(),
            answers: Vec::new(),
            authorities: Vec::new(),
            resources: Vec::new(),
        }
    }

    pub fn from_buffer(buffer: &mut BytePacketBuffer) -> Result<Self> {
        let mut result = Self::new();
        result.header.read(buffer);
        for _ in 0..result.header.questions {
            let mut question = Question::new("".to_string(), QueryType::UNKNOWN(0));
            question.read(buffer)?;
            result.questions.push(question);
        }

        for _ in 0..result.header.answers {
            let record = DnsRecord::read(buffer)?;
            result.answers.push(record);
        }

        for _ in 0..result.header.authoritative_entries {
            let record = DnsRecord::read(buffer)?;
            result.authorities.push(record);
        }

        for _ in 0..result.header.resource_entries {
            let record = DnsRecord::read(buffer)?;
            result.resources.push(record);
        }

        Ok(result)
    }

    pub fn write(&mut self, buffer: &mut BytePacketBuffer) -> Result<()> {
        self.header.questions = self.questions.len() as u16;
        self.header.answers = self.answers.len() as u16;
        self.header.authoritative_entries = self.authorities.len() as u16;
        self.header.resource_entries = self.resources.len() as u16;

        self.header.write(buffer)?;

        for v in &self.questions {
            v.write(buffer)?;
        }
        for v in &self.answers {
            v.write(buffer)?;
        }
        for v in &self.authorities {
            v.write(buffer)?;
        }
        for v in &self.resources {
            v.write(buffer)?;
        }

        Ok(())
    }

    pub fn get_random_a(&self) -> Option<Ipv4Addr> {
        self.answers
            .iter()
            .filter_map(|record| match record {
                DnsRecord::A { addr, .. } => Some(*addr),
                _ => None,
            })
            .next()
    }

    pub fn get_ns<'a>(&'a self, qname: &'a str) -> impl Iterator<Item = (&'a str, &'a str)> {
        self.authorities
            .iter()
            .filter_map(|record| match record {
                DnsRecord::NS { domain, host, .. } => Some((domain.as_str(), host.as_str())),
                _ => None,
            })
            .filter(move |(domain, _)| qname.ends_with(*domain))
    }

    pub fn get_resolved_ns(&self, qname: &str) -> Option<Ipv4Addr> {
        self.get_ns(qname)
            .flat_map(|(_, host)| {
                self.resources
                    .iter()
                    .filter_map(move |record| match record {
                        DnsRecord::A { domain, addr, .. } if domain == host => Some(addr),
                        _ => None,
                    })
            })
            .map(|addr| *addr)
            .next()
    }

    pub fn get_unresolved_ns<'a>(&'a self, qname: &'a str) -> Option<&'a str> {
        self.get_ns(qname).map(|(_, host)| host).next()
    }
}
