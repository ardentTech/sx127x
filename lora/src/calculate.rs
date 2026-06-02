use sx127x_common::Hz;
use crate::constants::HF_MIN_HZ;

const RSSI_LF_CONSTANT: i16 = -164; // TODO see p87 note2
const RSSI_HF_CONSTANT: i16 = -157; // TODO see p87 note2

pub(crate) fn data_rate(symbol_rate: f32, spreading_factor: f32, coding_rate: f32) -> u16 {
    (symbol_rate * spreading_factor * coding_rate) as u16
}

pub(crate) fn fei_hz(fei: i32, bandwidth_khz: f32) -> f64 {
    ((fei * 2i32.pow(24) / (32 * 10i32.pow(6))) as f64) * ((bandwidth_khz / 500f32) as f64)
}

pub(crate) fn fei_ppm(hz: f64, frf: u32) -> f64 {
    hz * (10u32.pow(6) / frf) as f64
}

pub(crate) fn ocp_trim(imax: u8) -> u8 {
    if imax == 0 {
        0
    } else if imax <= 120 {
        (imax - 45) / 5
    } else if imax <= 240 {
        (((imax as u16) + 30) / 10) as u8
    } else {
        27
    }
}

pub(crate) fn rssi_constant(frequency: Hz) -> i16 {
    if frequency >= HF_MIN_HZ { RSSI_HF_CONSTANT } else { RSSI_LF_CONSTANT }
}

pub(crate) fn rssi_dbm(frequency: Hz, rssi: i16) -> i16 {
    rssi_constant(frequency) + rssi
}

pub(crate) fn last_packet_rssi_dbm(
    frequency: Hz,
    last_packet_rssi: i16,
    last_packet_snr: i16,
    rssi: i16,
) -> i16 {
    if last_packet_snr >= 0 {
        rssi_dbm(frequency, rssi * 16 / 15) // see datasheet page 87 note 3
    } else {
        rssi_dbm(frequency, last_packet_rssi) + last_packet_snr
    }
}

/// Calculates the symbol period (Ts) in milliseconds.
///
/// See: datasheet section 4.1.1.7
pub(crate) fn symbol_period(symbol_rate: f32) -> f32 {
    (1f32 / symbol_rate) * 1000f32
}

/// Calculates the symbol rate (Rs)
///
/// See: datasheet section 4.1.1.5
pub(crate) fn symbol_rate(bandwidth: u32, spreading_factor: u32) -> f32 {
    bandwidth as f32 / 2u32.pow(spreading_factor) as f32
}

#[cfg(test)]
mod tests {
    use approx::assert_relative_eq;
    use super::*;

    #[test]
    fn data_rate_ok() {
        let res = data_rate(1953f32, 6f32, 0.8f32);
        assert_eq!(res, 9374u16);
    }

    #[test]
    fn fei_new_neg_fei_hz_ok() {
        let res = fei_hz(-2i32, 16f32);
        assert_relative_eq!(res, -0.032, epsilon=1e-3);
    }

    #[test]
    fn fei_new_pos_fei_hz_ok() {
        let res = fei_hz(8i32, 16f32);
        assert_relative_eq!(res, 0.128, epsilon=1e-3);
    }

    #[test]
    fn fei_new_neg_fei_ppm_ok() {
        let fei_hz = fei_hz(-4i32, 16f32);
        let fei_ppm = fei_ppm(fei_hz, 32u32);
        assert_relative_eq!(fei_ppm, -2000.0, epsilon=1e-3);
    }

    #[test]
    fn fei_new_pos_fei_ppm_ok() {
        let fei_hz = fei_hz(8i32, 16f32);
        let fei_ppm = fei_ppm(fei_hz, 32u32);
        assert_relative_eq!(fei_ppm, 4000.0, epsilon=1e-3);
    }

    #[test]
    fn last_packet_rssi_dbm_snr_negative() {
        assert_eq!(last_packet_rssi_dbm(HF_MIN_HZ - 1, 46, -2, 42), -120);
    }

    #[test]
    fn last_packet_rssi_dbm_snr_positive() {
        assert_eq!(last_packet_rssi_dbm(HF_MIN_HZ, 46, 10, 42), -113);
    }

    #[test]
    fn ocp_trim_imax_0() {
        let res = ocp_trim(0);
        assert_eq!(res, 0);
    }

    #[test]
    fn ocp_trim_imax_45() {
        let res = ocp_trim(45);
        assert_eq!(res, 0);
    }

    #[test]
    fn ocp_trim_imax_50() {
        let res = ocp_trim(50);
        assert_eq!(res, 1);
    }

    #[test]
    fn ocp_trim_imax_120() {
        let res = ocp_trim(120);
        assert_eq!(res, 15);
    }

    #[test]
    fn ocp_trim_imax_125() {
        let res = ocp_trim(125);
        assert_eq!(res, 15);
    }

    #[test]
    fn ocp_trim_imax_130() {
        let res = ocp_trim(130);
        assert_eq!(res, 16);
    }

    #[test]
    fn ocp_trim_imax_230() {
        let res = ocp_trim(230);
        assert_eq!(res, 26);
    }

    #[test]
    fn ocp_trim_imax_240() {
        let res = ocp_trim(240);
        assert_eq!(res, 27);
    }

    #[test]
    fn rssi_constant_hf() {
        assert_eq!(rssi_constant(HF_MIN_HZ), -157);
    }

    #[test]
    fn rssi_constant_lf() {
        assert_eq!(rssi_constant(HF_MIN_HZ - 1), -164);
    }

    #[test]
    fn rssi_dbm_hf() {
        assert_eq!(rssi_dbm(HF_MIN_HZ, 42), -115);
    }

    #[test]
    fn rssi_dbm_lf() {
        assert_eq!(rssi_dbm(HF_MIN_HZ - 1, 42), -122);
    }

    #[test]
    fn symbol_period_ok() {
        assert_relative_eq!(symbol_period(976.562), 1.024, epsilon=1e-3);
    }

    #[test]
    fn symbol_rate_ok() {
        let bandwidth = 125_000u32;
        let spreading_factor = 7u32;
        let symbol_rate = symbol_rate(bandwidth, spreading_factor);
        assert_relative_eq!(symbol_rate, 976.562, epsilon=1e-3);
    }
}