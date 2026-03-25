pub(crate) const fn get_bits(byte: u8, mask: u8, lsb_offset: u8) -> u8 {
    (byte & mask) >> lsb_offset
}

pub(crate) const fn set_bits(byte: &mut u8, bits: u8, mask: u8, lsb_offset: u8) {
    *byte &= !mask;
    *byte |= (bits << lsb_offset) & mask
}