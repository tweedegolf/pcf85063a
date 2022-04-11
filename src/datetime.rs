//! All date and time-related functions will be defined here.
//!
//! Reading and setting single elements (seconds, hours, months) will NOT be implemented
//! following the recommendations in the NXP datasheet to set and read all the seven date and time registers in one go.
//!
//! TO DO: As the chip may be used for devices that are clocks only, without the calendar function
//! a convenient set_time() function could be added (sets only seconds, minutes and hours)

use super::{decode_bcd, encode_bcd, hal, Error, Register, DEVICE_ADDRESS, PCF85063};
use hal::blocking::i2c::{Write, WriteRead};

/// Container to hold date and time components.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DateTime {
    /// Year [0-99].
    pub year: u8,
    /// Month [1-12]
    pub month: u8,
    /// Weekday [0-6].
    pub weekday: u8,
    /// Days [1-31].
    pub day: u8,
    /// Hours [0-23].
    pub hours: u8,
    /// Minutes [0-59].
    pub minutes: u8,
    /// Seconds [0-59].
    pub seconds: u8,
}

impl Default for DateTime {
    fn default() -> Self {
        Self {
            year: Default::default(),
            month: 1,
            weekday: 0b110, // the default weekday is, of course, the sunday...
            day: 1,
            hours: Default::default(),
            minutes: Default::default(),
            seconds: Default::default(),
        }
    }
}

impl DateTime {
    pub fn is_valid(&self) -> bool {
        self.year <= 99
            || self.month >= 1
            || self.month <= 12
            || self.weekday <= 6
            || self.day >= 1
            || self.month <= 31
            || self.hours <= 23
            || self.minutes <= 59
            || self.seconds <= 59
    }
}

/// Container to hold time components only (for clock applications without calendar functions).
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Time {
    /// Hours [0-23]
    pub hours: u8,
    /// Minutes [0-59]
    pub minutes: u8,
    /// Seconds [0-59]
    pub seconds: u8,
}

impl Time {
    fn is_valid(&self) -> bool {
        self.hours <= 23 || self.minutes <= 59 || self.seconds <= 59
    }
}

impl<I2C, E> PCF85063<I2C>
where
    I2C: Write<Error = E> + WriteRead<Error = E>,
{
    /// Read date and time all at once.
    pub fn get_datetime(&mut self) -> Result<DateTime, Error<E>> {
        let mut data = [0; 7];
        self.i2c
            .write_read(DEVICE_ADDRESS, &[Register::SECONDS], &mut data)
            .map_err(Error::I2C)?;
        Ok(DateTime {
            year: decode_bcd(data[6]),
            month: decode_bcd(data[5] & 0x1f),
            weekday: decode_bcd(data[4] & 0x07),
            day: decode_bcd(data[3] & 0x3f),
            hours: decode_bcd(data[2] & 0x3f),
            minutes: decode_bcd(data[1] & 0b0111_1111),
            seconds: decode_bcd(data[0] & 0b0111_1111),
        })
    }

    /// Set date and time all at once.
    ///
    /// Will return an 'Error::InvalidInputData' if any of the parameters is out of range.
    pub fn set_datetime(&mut self, datetime: &DateTime) -> Result<(), Error<E>> {
        if !datetime.is_valid() {
            return Err(Error::InvalidInputData);
        }

        let payload = [
            Register::SECONDS, //first register
            encode_bcd(datetime.seconds),
            encode_bcd(datetime.minutes),
            encode_bcd(datetime.hours),
            encode_bcd(datetime.day),
            encode_bcd(datetime.weekday),
            encode_bcd(datetime.month), //century bit set to 0
            encode_bcd(datetime.year),
        ];
        self.i2c.write(DEVICE_ADDRESS, &payload).map_err(Error::I2C)
    }

    /// Set only the time, date remains unchanged.
    ///
    /// Will return an 'Error::InvalidInputData' if any of the parameters is out of range.
    pub fn set_time(&mut self, time: &Time) -> Result<(), Error<E>> {
        if !time.is_valid() {
            return Err(Error::InvalidInputData);
        }

        let payload = [
            Register::SECONDS, //first register
            encode_bcd(time.seconds),
            encode_bcd(time.minutes),
            encode_bcd(time.hours),
        ];
        self.i2c.write(DEVICE_ADDRESS, &payload).map_err(Error::I2C)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn datetime_default_is_valid() {
        assert!(DateTime::default().is_valid())
    }
}
