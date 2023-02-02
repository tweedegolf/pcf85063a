#![no_std]

mod alarm;
mod datetime;

use embedded_hal_async::i2c::I2c;

/// All possible errors in this crate
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Error<E> {
    /// I2C bus error
    I2C(E),
    /// Invalid input data
    InvalidInputData,
    /// A time component was out of range
    ComponentRange,
}

impl<E> From<time::error::ComponentRange> for Error<E> {
    fn from(_: time::error::ComponentRange) -> Self {
        Self::ComponentRange
    }
}

pub struct Register;

impl Register {
    // control and status registers
    pub const CONTROL_1: u8 = 0x00;
    pub const CONTROL_2: u8 = 0x01;
    pub const OFFSET: u8 = 0x02;
    pub const RAM_BYTE: u8 = 0x03;

    // time and date registers
    pub const SECONDS: u8 = 0x04;
    pub const MINUTES: u8 = 0x05;
    pub const HOURS: u8 = 0x06;
    pub const DAYS: u8 = 0x07;
    pub const WEEKDAYS: u8 = 0x08;
    pub const MONTHS: u8 = 0x09;
    pub const YEARS: u8 = 0x0A;

    // alarm registers
    pub const SECOND_ALARM: u8 = 0x0B;
    pub const MINUTE_ALARM: u8 = 0x0C;
    pub const HOUR_ALARM: u8 = 0x0D;
    pub const DAY_ALARM: u8 = 0x0E;
    pub const WEEKDAY_ALARM: u8 = 0x0F;

    // timer registers
    pub const TIMER_VALUE: u8 = 0x10;
    pub const TIMER_MODE: u8 = 0x11;
}

pub struct BitFlags;

impl BitFlags {
    // control 1
    pub const CAP_SEL: u8 = 0b0000_0001; // internal oscillator capacitor selection
    pub const MODE_12_24: u8 = 0b0000_0010; // 12 or 24-hour mode
    pub const CIE: u8 = 0b0000_0100; // connection interrupt enable
                                 // 3: UNUSED
    pub const SR: u8 = 0b0001_0000; // software reset
    pub const STOP: u8 = 0b0010_0000; // RTC clock stop bit
                                  // 6: UNUSED
    pub const EXT_TEST: u8 = 0b1000_0000; // external clock test mode

    // control 2
    pub const COF: u8 = 0b0000_0111; // clkout control
    pub const TF: u8 = 0b0000_1000; // timer flag
    pub const HMI: u8 = 0b0001_0000; // half minute interrupt
    pub const MI: u8 = 0b0010_0000; // minute interrupt
    pub const AF: u8 = 0b0100_0000; // alarm flag
    pub const AIE: u8 = 0b1000_0000; // alarm interrupt enabled

    pub const AE: u8 = 0b1000_0000; // alarm enable/disable for all five (s/m/h/d/wd) settings
}

const DEVICE_ADDRESS: u8 = 0b1010001;

/// Two possible choices, used for various enable/disable bit flags
#[allow(non_camel_case_types)]
#[derive(Copy, Clone, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Control {
    /// Enable some feature, eg. timer
    On,
    /// Disable some feature, eg. timer
    Off,
}

/// PCF8563 driver
#[derive(Debug, Default)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct PCF85063<I2C> {
    /// The concrete I2C device implementation.
    i2c: I2C,
}

impl<I2C, E> PCF85063<I2C>
where
    I2C: I2c<Error = E>,
{
    /// Create a new instance of the PCF8563 driver.
    pub fn new(i2c: I2C) -> Self {
        PCF85063 { i2c }
    }

    /// Reset the RTC
    pub async fn reset(&mut self) -> Result<(), Error<E>> {
        self.set_register_bit_flag(Register::CONTROL_1, BitFlags::SR).await
    }

    /// Destroy driver instance, return I2C bus instance.
    pub fn destroy(self) -> I2C {
        self.i2c
    }

    /// Write to a register.
    pub async fn write_register(&mut self, register: u8, data: u8) -> Result<(), Error<E>> {
        let payload: [u8; 2] = [register, data];
        self.i2c
            .write(DEVICE_ADDRESS, &payload)
            .await
            .map_err(Error::I2C)
    }

    /// Read from a register.
    pub async fn read_register(&mut self, register: u8) -> Result<u8, Error<E>> {
        let mut data = [0];
        self.i2c
            .write_read(DEVICE_ADDRESS, &[register], &mut data)
            .await
            .map_err(Error::I2C)
            .and(Ok(data[0]))
    }

    /// Check if specific bits are set.
    pub async fn is_register_bit_flag_high(&mut self, address: u8, bitmask: u8) -> Result<bool, Error<E>> {
        let data = self.read_register(address).await?;
        Ok((data & bitmask) != 0)
    }

    /// Set specific bits.
    pub async fn set_register_bit_flag(&mut self, address: u8, bitmask: u8) -> Result<(), Error<E>> {
        let data = self.read_register(address).await?;
        if (data & bitmask) == 0 {
            self.write_register(address, data | bitmask).await
        } else {
            Ok(())
        }
    }

    /// Clear specific bits.
    pub async fn clear_register_bit_flag(&mut self, address: u8, bitmask: u8) -> Result<(), Error<E>> {
        let data = self.read_register(address).await?;
        if (data & bitmask) != 0 {
            self.write_register(address, data & !bitmask).await
        } else {
            Ok(())
        }
    }
}

impl<I2C, E> PCF85063<I2C>
where
    I2C: I2c<Error = E>,
{
    pub async fn read_ram_byte(&mut self) -> Result<u8, Error<E>> {
        self.read_register(Register::RAM_BYTE).await
    }

    pub async fn write_ram_byte(&mut self, byte: u8) -> Result<(), Error<E>> {
        self.write_register(Register::RAM_BYTE, byte).await
    }
}

impl<I2C, E> PCF85063<I2C>
where
    I2C: I2c<Error = E>,
{
    pub async fn stop_clock(&mut self) -> Result<(), Error<E>> {
        self.set_register_bit_flag(Register::CONTROL_1, BitFlags::STOP).await
    }

    pub async fn start_clock(&mut self) -> Result<(), Error<E>> {
        self.clear_register_bit_flag(Register::CONTROL_1, BitFlags::STOP).await
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
pub enum OutputFrequency {
    Hz32768 = 0b000,
    Hz16384 = 0b001,
    Hz8192 = 0b010,
    Hz4096 = 0b011,
    Hz2048 = 0b100,
    Hz1024 = 0b101,
    Hz1 = 0b110,
    Hz0 = 0b111,
}

impl Default for OutputFrequency {
    fn default() -> Self {
        OutputFrequency::Hz32768
    }
}

impl OutputFrequency {
    pub const fn bits(self) -> u8 {
        self as u8
    }
}

impl<I2C, E> PCF85063<I2C>
where
    I2C: I2c<Error = E>,
{
    pub async fn read_clock_output_frequency(&mut self) -> Result<OutputFrequency, Error<E>> {
        let value = self.read_register(Register::CONTROL_2).await? & BitFlags::COF;

        Ok(unsafe { core::mem::transmute(value) })
    }

    pub async fn write_clock_output_frequency(&mut self, freq: OutputFrequency) -> Result<(), Error<E>> {
        let value = self.read_register(Register::CONTROL_2).await?;
        let cleared = value ^ BitFlags::COF;
        let set = cleared | freq as u8;

        self.write_register(Register::CONTROL_2, set).await
    }
}

/// Convert the Binary Coded Decimal value to decimal (only the lowest 7 bits).
fn decode_bcd(input: u8) -> u8 {
    let digits: u8 = input & 0xf;
    let tens: u8 = (input >> 4) & 0x7;
    10 * tens + digits
}

/// Convert the decimal value to Binary Coded Decimal.
fn encode_bcd(input: u8) -> u8 {
    let digits: u8 = input % 10;
    let tens: u8 = input / 10;
    let tens = tens << 4;
    tens + digits
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_convert_decode_bcd() {
        assert_eq!(0, decode_bcd(0b0000_0000));
        assert_eq!(1, decode_bcd(0b0000_0001));
        assert_eq!(9, decode_bcd(0b0000_1001));
        assert_eq!(10, decode_bcd(0b0001_0000));
        assert_eq!(11, decode_bcd(0b0001_0001));
        assert_eq!(19, decode_bcd(0b0001_1001));
        assert_eq!(20, decode_bcd(0b0010_0000));
        assert_eq!(21, decode_bcd(0b0010_0001));
        assert_eq!(59, decode_bcd(0b0101_1001));
    }

    #[test]
    fn can_convert_encode_bcd() {
        assert_eq!(0b0000_0000, encode_bcd(0));
        assert_eq!(0b0000_0001, encode_bcd(1));
        assert_eq!(0b0000_1001, encode_bcd(9));
        assert_eq!(0b0001_0000, encode_bcd(10));
        assert_eq!(0b0001_0001, encode_bcd(11));
        assert_eq!(0b0001_1001, encode_bcd(19));
        assert_eq!(0b0010_0000, encode_bcd(20));
        assert_eq!(0b0010_0001, encode_bcd(21));
        assert_eq!(0b0101_1001, encode_bcd(59));
    }
}
