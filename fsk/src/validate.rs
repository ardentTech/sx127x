const FDEV_MIN_HZ: u16 = 600;
const FDEV_MAX_HZ: u32 = 200_000;
const FIFO_THRESHOLD_MAX: u8 = 63;
const PREAMBLE_DETECTOR_THRESHOLD_MAX: u8 = 31;
const RSSI_OFFSET_MIN: i8 = -16;
const RSSI_OFFSET_MAX: i8 = 15;
const SYNC_SIZE_MAX: u8 = 7;

pub(crate) fn fdev(hz: u32) -> bool {
    hz >= FDEV_MIN_HZ as u32 && hz <= FDEV_MAX_HZ
}

pub(crate) fn fifo_threshold(threshold: u8) -> bool {
    threshold <= FIFO_THRESHOLD_MAX
}

pub(crate) fn preamble_detector_threshold(threshold: u8) -> bool {
    threshold <= PREAMBLE_DETECTOR_THRESHOLD_MAX
}

pub(crate) fn rssi_offset(offset: i8) -> bool {
    offset >= RSSI_OFFSET_MIN && offset <= RSSI_OFFSET_MAX
}

pub(crate) fn sync_size(size: u8) -> bool {
    size <= SYNC_SIZE_MAX
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fdev_low() {
        assert!(!fdev(FDEV_MIN_HZ as u32 - 1));
    }

    #[test]
    fn fdev_high() {
        assert!(!fdev(FDEV_MAX_HZ + 1));
    }

    #[test]
    fn fdev_ok() {
        assert!(fdev(FDEV_MAX_HZ - 1));
    }

    #[test]
    fn fifo_threshold_high() {
        assert!(!fifo_threshold(FIFO_THRESHOLD_MAX + 1));
    }

    #[test]
    fn fifo_threshold_ok() {
        assert!(fifo_threshold(FIFO_THRESHOLD_MAX - 1));
    }

    #[test]
    fn preamble_detector_threshold_high() {
        assert!(!preamble_detector_threshold(PREAMBLE_DETECTOR_THRESHOLD_MAX + 1));
    }

    #[test]
    fn preamble_detector_threshold_ok() {
        assert!(preamble_detector_threshold(PREAMBLE_DETECTOR_THRESHOLD_MAX - 1));
    }

    #[test]
    fn rssi_offset_low() {
        assert!(!rssi_offset(RSSI_OFFSET_MIN - 1));
    }

    #[test]
    fn rssi_offset_high() {
        assert!(!rssi_offset(RSSI_OFFSET_MAX + 1));
    }

    #[test]
    fn rssi_offset_ok() {
        assert!(rssi_offset(RSSI_OFFSET_MIN + 1));
    }

    #[test]
    fn sync_size_high() {
        assert!(!sync_size(SYNC_SIZE_MAX + 1));
    }

    #[test]
    fn sync_size_ok() {
        assert!(sync_size(SYNC_SIZE_MAX));
    }
}