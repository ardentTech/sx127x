const BOOST_POWER_MIN: u8 = 2;
const BOOST_POWER_MAX: u8 = 20;
const RFO_POWER_MAX: u8 = 15;
const PREAMBLE_LENGTH_MIN: u16 = 6;
pub const RX_TIMEOUT_SYMBOLS_MIN: u16 = 4;
pub const RX_TIMEOUT_SYMBOLS_MAX: u16 = 1023;

pub(crate) fn boost_power(power: u8) -> bool {
    power >= BOOST_POWER_MIN && power <= BOOST_POWER_MAX
}


pub(crate) fn preamble_length(length: u16) -> bool {
    length >= PREAMBLE_LENGTH_MIN
}

pub(crate) fn rfo_power(power: u8) -> bool {
    power <= RFO_POWER_MAX
}

pub(crate) fn rx_timeout_symbols(symbols: u16) -> bool {
    symbols >= RX_TIMEOUT_SYMBOLS_MIN && symbols <= RX_TIMEOUT_SYMBOLS_MAX
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn boost_power_low() {
        assert!(!boost_power(BOOST_POWER_MIN - 1));
    }

    #[test]
    fn boost_power_high() {
        assert!(!boost_power(BOOST_POWER_MAX + 1))
    }

    #[test]
    fn boost_power_ok() {
        assert!(boost_power(BOOST_POWER_MIN));
    }

    #[test]
    fn preamble_length_low() {
        assert!(!preamble_length(PREAMBLE_LENGTH_MIN - 1));
    }

    #[test]
    fn preamble_length_ok() {
        assert!(preamble_length(PREAMBLE_LENGTH_MIN));
    }

    #[test]
    fn rfo_power_high() {
        assert!(!rfo_power(RFO_POWER_MAX + 1))
    }

    #[test]
    fn rfo_power_ok() {
        assert!(rfo_power(RFO_POWER_MAX));
    }

    #[test]
    fn rx_timeout_symbols_low() {
        assert!(!rx_timeout_symbols(RX_TIMEOUT_SYMBOLS_MIN - 1));
    }

    #[test]
    fn rx_timeout_symbols_high() {
        assert!(!rx_timeout_symbols(RX_TIMEOUT_SYMBOLS_MAX + 1));
    }

    #[test]
    fn rx_timeout_symbols_ok() {
        assert!(rx_timeout_symbols(RX_TIMEOUT_SYMBOLS_MIN + 1));
    }
}