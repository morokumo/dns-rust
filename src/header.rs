use crate::{buffer::BytePacketBuffer, enums::ResultCode};

type Error = Box<dyn std::error::Error>;
type Result<T> = std::result::Result<T, Error>;
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
    pub fn write(&self, buffer: &mut BytePacketBuffer) -> Result<()> {
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