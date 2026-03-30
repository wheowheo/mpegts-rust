pub struct BitReader<'a> {
    data: &'a [u8],
    byte_pos: usize,
    bit_pos: u8,
}

impl<'a> BitReader<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self { data, byte_pos: 0, bit_pos: 0 }
    }

    pub fn bits_left(&self) -> usize {
        if self.byte_pos >= self.data.len() { return 0; }
        (self.data.len() - self.byte_pos) * 8 - self.bit_pos as usize
    }

    pub fn read_bits(&mut self, n: u8) -> Option<u32> {
        if n == 0 { return Some(0); }
        if n > 32 { return None; }
        let mut val = 0u32;
        for _ in 0..n {
            if self.byte_pos >= self.data.len() { return None; }
            val = (val << 1) | ((self.data[self.byte_pos] >> (7 - self.bit_pos)) & 1) as u32;
            self.bit_pos += 1;
            if self.bit_pos >= 8 {
                self.bit_pos = 0;
                self.byte_pos += 1;
            }
        }
        Some(val)
    }

    pub fn read_bit(&mut self) -> Option<bool> {
        self.read_bits(1).map(|v| v != 0)
    }

    pub fn read_ue(&mut self) -> Option<u32> {
        let mut leading_zeros = 0u8;
        loop {
            match self.read_bit()? {
                false => leading_zeros += 1,
                true => break,
            }
            if leading_zeros > 31 { return None; }
        }
        if leading_zeros == 0 { return Some(0); }
        let suffix = self.read_bits(leading_zeros)?;
        Some((1 << leading_zeros) - 1 + suffix)
    }

    pub fn read_se(&mut self) -> Option<i32> {
        let val = self.read_ue()?;
        if val % 2 == 0 {
            Some(-(val as i32 / 2))
        } else {
            Some((val as i32 + 1) / 2)
        }
    }

    pub fn skip(&mut self, n: usize) {
        for _ in 0..n {
            if self.read_bit().is_none() { break; }
        }
    }
}
