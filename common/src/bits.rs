// TODO determine LSB offset from mask?
pub const fn get_bits(byte: u8, mask: u8, lsb_offset: u8) -> u8 {
    (byte & mask) >> lsb_offset
}

pub const fn set_bits(byte: &mut u8, bits: u8, mask: u8, lsb_offset: u8) {
    unset_bits(byte, mask);
    *byte |= (bits << lsb_offset) & mask
}

pub const fn unset_bits(byte: &mut u8, mask: u8) {
    *byte &= !mask
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_bits_ok() {
        let byte = 0b1010_1010;
        let mask = 0b0011_1000;
        let lsb_offset = 0x3;
        assert_eq!(get_bits(byte, mask, lsb_offset), 0b101);
    }

    #[test]
    fn set_bits_ok() {
        let mut byte = 0b1010_1010;
        let mask = 0b0011_1000;
        let lsb_offset = 0x3;
        set_bits(&mut byte, 0b010, mask, lsb_offset);
        assert_eq!(byte, 0b1001_0010);
    }

    #[test]
    fn unset_bits_ok() {
        let mut byte = 0b1010_1010;
        let mask = 0b0111_0000;
        unset_bits(&mut byte, mask);
        assert_eq!(byte, 0b1000_1010);
    }
}