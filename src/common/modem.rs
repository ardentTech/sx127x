use embedded_hal_async::spi::SpiDevice;
use crate::lora::driver::Sx127xLoraError;
use crate::lora::types::DeviceMode;

trait Modem<SPI: SpiDevice> {
    /// Sets the device mode.
    ///
    /// See: datasheet section 2.1.4
    async fn set_device_mode(&mut self, device_mode: DeviceMode) -> Result<(), Sx127xLoraError<SPI::Error>> {
        // let mut byte = self.read(OP_MODE).await?;
        // set_bits(&mut byte, device_mode as u8, OP_MODE_MODE_MASK, 0);
        // self.write(OP_MODE, byte).await
    }
}