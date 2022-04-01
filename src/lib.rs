#![no_std]

use embedded_hal as hal;

use hal::blocking::i2c::{Write, WriteRead};

/// All possible errors in this crate
#[derive(Debug)]
pub enum Error<E> {
    /// I2C bus error
    I2C(E),
    /// Invalid input data
    InvalidInputData,
}
struct Register;

impl Register {
    // control and status registers
    const CONTROL_1: u8 = 0x00;
    const CONTROL_2: u8 = 0x01;
    const OFFSET: u8 = 0x02;
    const RAM_BYTE: u8 = 0x03;

    // time and date registers
    const SECONDS: u8 = 0x04;
    const MINUTES: u8 = 0x05;
    const HOURS: u8 = 0x06;
    const DAYS: u8 = 0x07;
    const WEEKDAYS: u8 = 0x08;
    const MONTHS: u8 = 0x09;
    const YEARS: u8 = 0x0A;

    // alarm registers
    const SECOND_ALARM: u8 = 0x0B;
    const MINUTE_ALARM: u8 = 0x0C;
    const HOUR_ALARM: u8 = 0x0D;
    const DAY_ALARM: u8 = 0x0E;
    const WEEKDAY_ALARM: u8 = 0x0F;

    // timer registers
    const TIMER_VALUE: u8 = 0x10;
    const TIMER_MODE: u8 = 0x11;
}

struct BitFlags;

impl BitFlags {
    const TEST1: u8 = 0b1000_0000;
}

const DEVICE_ADDRESS: u8 = 0b1010001;

/// Two possible choices, used for various enable/disable bit flags
#[allow(non_camel_case_types)]
#[derive(Copy, Clone, Debug)]
pub enum Control {
    /// Enable some feature, eg. timer
    On,
    /// Disable some feature, eg. timer
    Off,
}

/// PCF8563 driver
#[derive(Debug, Default)]
pub struct PCF8563<I2C> {
    /// The concrete I2C device implementation.
    i2c: I2C,
}

impl<I2C, E> PCF8563<I2C>
where
    I2C: Write<Error = E> + WriteRead<Error = E>,
{
    /// Create a new instance of the PCF8563 driver.
    pub fn new(i2c: I2C) -> Self {
        PCF8563 { i2c }
    }

    /// Destroy driver instance, return I2C bus instance.
    pub fn destroy(self) -> I2C {
        self.i2c
    }

    /// Write to a register.
    fn write_register(&mut self, register: u8, data: u8) -> Result<(), Error<E>> {
        let payload: [u8; 2] = [register, data];
        self.i2c.write(DEVICE_ADDRESS, &payload).map_err(Error::I2C)
    }

    /// Read from a register.
    fn read_register(&mut self, register: u8) -> Result<u8, Error<E>> {
        let mut data = [0];
        self.i2c
            .write_read(DEVICE_ADDRESS, &[register], &mut data)
            .map_err(Error::I2C)
            .and(Ok(data[0]))
    }

    /// Check if specific bits are set.
    fn is_register_bit_flag_high(&mut self, address: u8, bitmask: u8) -> Result<bool, Error<E>> {
        let data = self.read_register(address)?;
        Ok((data & bitmask) != 0)
    }

    /// Set specific bits.
    fn set_register_bit_flag(&mut self, address: u8, bitmask: u8) -> Result<(), Error<E>> {
        let data = self.read_register(address)?;
        if (data & bitmask) == 0 {
            self.write_register(address, data | bitmask)
        } else {
            Ok(())
        }
    }

    /// Clear specific bits.
    fn clear_register_bit_flag(&mut self, address: u8, bitmask: u8) -> Result<(), Error<E>> {
        let data = self.read_register(address)?;
        if (data & bitmask) != 0 {
            self.write_register(address, data & !bitmask)
        } else {
            Ok(())
        }
    }
}
