use crate::PACKET_BUFFER_SIZE;

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

    pub fn pos(&self) -> usize {
        self.pos
    }

    pub fn step(&mut self, steps: usize) -> Result<()> {
        self.pos += steps;
        Ok(())
    }

    pub fn seek(&mut self, pos: usize) -> Result<()> {
        self.pos = pos;
        Ok(())
    }

    pub fn read(&mut self) -> Result<u8> {
        if self.pos >= PACKET_BUFFER_SIZE {
            return Err("End of buffer.".into());
        }
        let res = self.buffer[self.pos];
        self.pos += 1;
        Ok(res)
    }
    pub fn read_u16(&mut self) -> Result<u16> {
        let res = ((self.read()? as u16) << 8) | (self.read()? as u16);
        Ok(res)
    }
    pub fn read_u32(&mut self) -> Result<u32> {
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
    pub fn write_u8(&mut self, val: u8) -> Result<()> {
        self.write(val)?;
        Ok(())
    }
    pub fn write_u16(&mut self, val: u16) -> Result<()> {
        self.write((val >> 8) as u8)?;
        self.write(((val >> 0) & 0xFF) as u8)?;
        Ok(())
    }
    pub fn write_u32(&mut self, val: u32) -> Result<()> {
        self.write(((val >> 24) & 0xFF) as u8)?;
        self.write(((val >> 16) & 0xFF) as u8)?;
        self.write(((val >> 8) & 0xFF) as u8)?;
        self.write(((val >> 0) & 0xFF) as u8)?;
        Ok(())
    }

    pub fn get(&mut self, pos: usize) -> Result<u8> {
        if pos >= PACKET_BUFFER_SIZE {
            return Err("End of buffer.".into());
        }
        Ok(self.buffer[pos])
    }

    pub fn get_range(&mut self, start: usize, len: usize) -> Result<&[u8]> {
        if start + len >= PACKET_BUFFER_SIZE {
            return Err("".into());
        }
        Ok(&self.buffer[start..start + len])
    }

    pub fn read_qname(&mut self, outstr: &mut String) -> Result<()> {
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

    pub fn write_qname(&mut self, qname: &str) -> Result<()> {
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

    fn set(&mut self, pos: usize, val: u8) -> Result<()> {
        self.buffer[pos] = val;
        Ok(())
    }

    pub fn set_u16(&mut self, pos: usize, val: u16) -> Result<()> {
        self.set(pos, (val >> 8) as u8)?;
        self.set(pos + 1, val as u8)?;
        Ok(())
    }
}