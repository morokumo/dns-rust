use std::{fs::File, net::Ipv4Addr, usize};

const PACKET_BUFFER_SIZE: usize = 512;
type Error = Box<dyn std::error::Error>;
type Result<T> = std::result::Result<T, Error>;

pub struct BytePacketBuffer {
    pub buffer: [u8; PACKET_BUFFER_SIZE],
    pub pos: usize,
}

impl BytePacketBuffer {
    pub fn new() -> Self {
        Self {
            buffer: [0; PACKET_BUFFER_SIZE],
            pos: 0,
        }
    }

    fn pos(&self) -> usize {
        self.pos
    }

    fn step(&mut self, steps: usize) -> Result<()> {
        self.pos += steps;
        Ok(())
    }

    fn seek(&mut self, pos: usize) -> Result<()> {
        self.pos = pos;
        Ok(())
    }

    fn read(&mut self) -> Result<u8> {
        if self.pos >= PACKET_BUFFER_SIZE {
            return Err("End of buffer.".into());
        }
        let res = self.buffer[self.pos];
        self.pos += 1;
        Ok(res)
    }
    fn read_u16(&mut self) -> Result<u16> {
        let res = ((self.read()? as u16) << 8) | (self.read()? as u16);
        Ok(res)
    }
    fn read_u32(&mut self) -> Result<u32> {
        let res = ((self.read()? as u32) << 24)
            | ((self.read()? as u32) << 16)
            | ((self.read()? as u32) << 8)
            | (self.read()? as u32);
        Ok(res)
    }

    fn write(&mut self, val: u8) -> Result<()> {
        if self.pos >= PACKET_BUFFER_SIZE {
            return Err("End of buffer.".into());
        }
        self.buffer[self.pos] = val;
        self.pos += 1;
        Ok(())
    }
    fn write_u8(&mut self, val: u8) -> Result<()> {
        self.write(val)?;
        Ok(())
    }
    fn write_u16(&mut self, val: u16) -> Result<()> {
        self.write((val >> 8) as u8)?;
        self.write(((val >> 0) & 0xFF) as u8)?;
        Ok(())
    }
    fn write_u32(&mut self, val: u32) -> Result<()> {
        self.write(((val >> 24) & 0xFF) as u8)?;
        self.write(((val >> 16) & 0xFF) as u8)?;
        self.write(((val >> 8) & 0xFF) as u8)?;
        self.write(((val >> 0) & 0xFF) as u8)?;
        Ok(())
    }

    fn get(&mut self, pos: usize) -> Result<u8> {
        if pos >= PACKET_BUFFER_SIZE {
            return Err("End of buffer.".into());
        }
        Ok(self.buffer[pos])
    }

    fn get_range(&mut self, start: usize, len: usize) -> Result<&[u8]> {
        if start + len >= PACKET_BUFFER_SIZE {
            return Err("".into());
        }
        Ok(&self.buffer[start..start + len])
    }

    fn read_qname(&mut self, outstr: &mut String) -> Result<()> {
        let mut pos = self.pos;

        let mut jumped = false;
        let max_jumps = 5;
        let mut jumps_performed = 0;

        let mut delim = "";
        loop {
            if jumps_performed > max_jumps {
                return Err(format!("Limit of {} jumps excceeded", max_jumps).into());
            }

            let len = self.get(pos)?;

            if (len & 0xc0) == 0xc0 {
                if !jumped {
                    self.seek(pos + 2)?;
                }
                let b2 = self.get(pos + 1)? as u16;
                let offset = (((len as u16) ^ 0xc0) << 0) | b2;
                pos = offset as usize;
                jumped = true;
                jumps_performed += 1;
                continue;
            } else {
                pos += 1;
                if len == 0 {
                    break;
                }
                outstr.push_str(delim);
                let str_buffer = self.get_range(pos, len as usize)?;
                outstr.push_str(&String::from_utf8_lossy(str_buffer).to_lowercase());
                delim = ".";
                pos += len as usize;
            }
        }

        if !jumped {
            self.seek(pos)?;
        }
        Ok(())
    }

    fn write_qname(&mut self, qname: &str) -> Result<()> {
        for label in qname.split('.') {
            let len = label.len();
            if len > 0x3f {
                return Err("Single lable exceeds 63 characlets of length".into());
            }
            self.write_u8(len as u8)?;
            for b in label.as_bytes() {
                self.write_u8(*b)?;
            }
        }
        self.write_u8(0)?;
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResultCode {
    NOERROR = 0,
    FORMERR = 1,
    SERVFAIL = 2,
    NXDOMAIN = 3,
    NOTIMP = 4,
    REFUSED = 5,
}

impl ResultCode {
    pub fn from_num(num: u8) -> ResultCode {
        match num {
            1 => ResultCode::FORMERR,
            2 => ResultCode::SERVFAIL,
            3 => ResultCode::NXDOMAIN,
            4 => ResultCode::NOTIMP,
            5 => ResultCode::REFUSED,
            0 | _ => ResultCode::NOERROR,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Header {
    pub id: u16,

    pub recursion_desired: bool,
    pub truncated_message: bool,
    pub authoritative_answer: bool,
    pub operation_code: u8,
    pub responce: bool,

    pub responce_code: ResultCode,
    pub checking_disabled: bool,
    pub authed_data: bool,
    pub z: bool,
    pub recursion_available: bool,

    pub questions: u16,
    pub answers: u16,
    pub authoritative_entries: u16,
    pub resource_entries: u16,
}

impl Header {
    pub fn new() -> Self {
        Self {
            id: 0,

            recursion_desired: false,
            truncated_message: false,
            authoritative_answer: false,
            operation_code: 0,
            responce: false,

            responce_code: ResultCode::NOERROR,
            checking_disabled: false,
            authed_data: false,
            z: false,
            recursion_available: false,

            questions: 0,
            answers: 0,
            authoritative_entries: 0,
            resource_entries: 0,
        }
    }

    pub fn read(&mut self, buffer: &mut BytePacketBuffer) -> Result<()> {
        self.id = buffer.read_u16()?;

        let flags = buffer.read_u16()?;
        let a = (flags >> 8) as u8;
        let b = (flags & 0xFF) as u8;
        self.recursion_desired = (a & (1 << 0)) > 0;
        self.truncated_message = (a & (1 << 1)) > 0;
        self.authoritative_answer = (a & (1 << 2)) > 0;
        self.operation_code = (a >> 3) & 0x0F;
        self.responce = (a & (1 << 7)) > 0;

        self.responce_code = ResultCode::from_num(b & 0x0F);
        self.checking_disabled = (b & (1 << 4)) > 0;
        self.authed_data = (b & (1 << 5)) > 0;
        self.z = (b & (1 << 6)) > 0;
        self.recursion_available = (b & (1 << 7)) > 0;

        self.questions = buffer.read_u16()?;
        self.answers = buffer.read_u16()?;
        self.authoritative_entries = buffer.read_u16()?;
        self.resource_entries = buffer.read_u16()?;

        Ok(())
    }
    fn write(&self, buffer: &mut BytePacketBuffer) -> Result<()> {
        buffer.write_u16(self.id)?;

        buffer.write_u8(
            (self.recursion_desired as u8)
                | ((self.truncated_message as u8) << 1)
                | ((self.authed_data as u8) << 2)
                | (self.operation_code << 3)
                | ((self.responce as u8) << 7) as u8,
        )?;

        buffer.write_u8(
            (self.responce_code as u8)
                | ((self.checking_disabled as u8) << 4)
                | ((self.authed_data as u8) << 5)
                | ((self.z as u8) << 6)
                | ((self.recursion_available as u8) << 7),
        )?;

        buffer.write_u16(self.questions)?;
        buffer.write_u16(self.answers)?;
        buffer.write_u16(self.authoritative_entries)?;
        buffer.write_u16(self.resource_entries)?;

        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum QueryType {
    UNKNOWN(u16),
    A,
}

impl QueryType {
    pub fn to_num(&self) -> u16 {
        match *self {
            QueryType::UNKNOWN(x) => x,
            QueryType::A => 1,
        }
    }
    pub fn from_num(num: u16) -> QueryType {
        match num {
            1 => QueryType::A,
            _ => QueryType::UNKNOWN(num),
        }
    }
}

#[derive(Debug)]
pub struct Question {
    pub name: String,
    pub qtype: QueryType,
}

impl Question {
    pub fn new(name: String, qtype: QueryType) -> Self {
        Self { name, qtype: qtype }
    }

    pub fn read(&mut self, buffer: &mut BytePacketBuffer) -> Result<()> {
        buffer.read_qname(&mut self.name)?;
        self.qtype = QueryType::from_num(buffer.read_u16()?);
        let _ = buffer.read_u16()?; // class

        Ok(())
    }

    pub fn write(&self, buffer: &mut BytePacketBuffer) -> Result<()> {
        buffer.write_qname(&self.name)?;
        buffer.write_u16(self.qtype.to_num())?;
        buffer.write_u16(1);
        Ok(())
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Record {
    UNKNOWN {
        domain: String,
        qtype: u16,
        data_len: u16,
        ttl: u32,
    },
    A {
        domain: String,
        addr: Ipv4Addr,
        ttl: u32,
    },
}

impl Record {
    fn read(buffer: &mut BytePacketBuffer) -> Result<Record> {
        let mut domain = String::new();
        buffer.read_qname(&mut domain)?;
        let qtype_num = buffer.read_u16()?;
        let qtype = QueryType::from_num(qtype_num);
        let _ = buffer.read_u16()?;
        let ttl = buffer.read_u32()?;
        let data_len = buffer.read_u16()?;
        match qtype {
            QueryType::A => {
                let raw_addr = buffer.read_u32()?;
                let addr = Ipv4Addr::new(
                    ((raw_addr >> 24) & 0xFF) as u8,
                    ((raw_addr >> 16) & 0xFF) as u8,
                    ((raw_addr >> 8) & 0xFF) as u8,
                    ((raw_addr >> 0) & 0xFF) as u8,
                );
                Ok(Record::A { domain, addr, ttl })
            }
            QueryType::UNKNOWN(_) => {
                buffer.step(data_len as usize)?;
                Ok(Record::UNKNOWN {
                    domain,
                    qtype: qtype.to_num(),
                    data_len,
                    ttl,
                })
            }
        }
    }
    fn write(&self, buffer: &mut BytePacketBuffer) -> Result<usize> {
        let start_pos = buffer.pos();
        match *self {
            Record::A {
                ref domain,
                ref addr,
                ttl,
            } => {
                buffer.write_qname(domain)?;
                buffer.write_u16(QueryType::A.to_num())?;
                buffer.write_u16(1)?;
                buffer.write_u32(ttl)?;
                buffer.write_u16(1)?;

                let octets = addr.octets();
                buffer.write_u8(octets[0])?;
                buffer.write_u8(octets[1])?;
                buffer.write_u8(octets[2])?;
                buffer.write_u8(octets[3])?;
            }
            Record::UNKNOWN { .. } => {}
        }
        Ok(buffer.pos() - start_pos)
    }
}

pub struct Packet {
    pub header: Header,
    pub questions: Vec<Question>,
    pub answers: Vec<Record>,
    pub authorities: Vec<Record>,
    pub resources: Vec<Record>,
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
            let record = Record::read(buffer)?;
            result.answers.push(record);
        }

        for _ in 0..result.header.authoritative_entries {
            let record = Record::read(buffer)?;
            result.authorities.push(record);
        }

        for _ in 0..result.header.resource_entries {
            let record = Record::read(buffer)?;
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
}
