/// Calculates the bite rate in b/s.
///
/// See: datasheet section 2.1.1
pub(crate) fn bit_rate(fxosc: f32, rate: f32, frac: f32) -> f32 {
    fxosc / (rate + (frac / 16f32))
}

#[cfg(test)]
mod tests {
    use approx::assert_relative_eq;
    use super::*;

    #[test]
    fn bit_rate_ok() {
        let fxosc = 32000000f32;
        let rate = 6667f32;
        let frac = 0f32;
        assert_relative_eq!(bit_rate(fxosc, rate, frac), 4799.760, epsilon=1e-3);
    }
}