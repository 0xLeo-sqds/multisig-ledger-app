pub mod inner;
pub mod solana_message;
pub mod squads;
pub mod vault_tx;

/// Zero-copy reader over a byte buffer. No allocation.
/// Ported from msig-cli SafeReader, adapted for no_std.
pub struct Reader<'a> {
    data: &'a [u8],
    pos: usize,
}

/// Parse error — Copy enum, messages in flash via &'static str.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParseError {
    /// Reached end of buffer unexpectedly.
    Eof { offset: usize, need: usize, have: usize },
    /// Discriminator did not match expected value.
    InvalidDiscriminator,
    /// A structural invariant was violated (e.g., index out of bounds).
    InvalidStructure,
    /// Boolean tag was neither 0 nor 1.
    InvalidTag { tag: u8 },
    /// Nesting too deep for stack safety.
    NestingTooDeep,
    /// Versioned message not supported.
    VersionedMessageNotSupported,
}

impl<'a> Reader<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self { data, pos: 0 }
    }

    pub fn remaining(&self) -> usize {
        self.data.len().saturating_sub(self.pos)
    }

    pub fn position(&self) -> usize {
        self.pos
    }

    pub fn read_bytes(&mut self, n: usize) -> Result<&'a [u8], ParseError> {
        if self.remaining() < n {
            return Err(ParseError::Eof {
                offset: self.pos,
                need: n,
                have: self.remaining(),
            });
        }
        let slice = &self.data[self.pos..self.pos + n];
        self.pos += n;
        Ok(slice)
    }

    pub fn skip(&mut self, n: usize) -> Result<(), ParseError> {
        if self.remaining() < n {
            return Err(ParseError::Eof {
                offset: self.pos,
                need: n,
                have: self.remaining(),
            });
        }
        self.pos += n;
        Ok(())
    }

    pub fn read_u8(&mut self) -> Result<u8, ParseError> {
        let bytes = self.read_bytes(1)?;
        Ok(bytes[0])
    }

    pub fn read_u16_le(&mut self) -> Result<u16, ParseError> {
        let bytes = self.read_bytes(2)?;
        Ok(u16::from_le_bytes([bytes[0], bytes[1]]))
    }

    pub fn read_u32_le(&mut self) -> Result<u32, ParseError> {
        let bytes = self.read_bytes(4)?;
        Ok(u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
    }

    pub fn read_u64_le(&mut self) -> Result<u64, ParseError> {
        let bytes = self.read_bytes(8)?;
        Ok(u64::from_le_bytes([
            bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
        ]))
    }

    pub fn read_pubkey(&mut self) -> Result<&'a [u8; 32], ParseError> {
        let bytes = self.read_bytes(32)?;
        // Safety: we know bytes.len() == 32
        Ok(bytes.try_into().map_err(|_| ParseError::InvalidStructure)?)
    }

    pub fn read_discriminator(&mut self, expected: &[u8; 8]) -> Result<(), ParseError> {
        let bytes = self.read_bytes(8)?;
        if bytes != expected {
            return Err(ParseError::InvalidDiscriminator);
        }
        Ok(())
    }

    /// Read a Solana compact-u16 encoded length.
    pub fn read_compact_u16(&mut self) -> Result<u16, ParseError> {
        let first = self.read_u8()? as u16;
        if first < 0x80 {
            return Ok(first);
        }
        let second = self.read_u8()? as u16;
        if second < 0x80 {
            return Ok((first & 0x7f) | (second << 7));
        }
        let third = self.read_u8()? as u16;
        Ok((first & 0x7f) | ((second & 0x7f) << 7) | (third << 14))
    }
}
