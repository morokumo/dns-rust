use crate::{buffer::BytePacketBuffer, enums::QueryType};

type Error = Box<dyn std::error::Error>;
type Result<T> = std::result::Result<T, Error>;
#[derive(Debug, Clone)]
pub struct Question {
    pub name: String,
    pub qtype: QueryType,
}

impl Question {
    pub fn new(name: String, qtype: QueryType) -> Self {
        Self { name, qtype }
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
        buffer.write_u16(1)?;
        Ok(())
    }
}
