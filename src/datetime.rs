//! All date and time-related functions will be defined here.
//!
//! Reading and setting single elements (seconds, hours, months) will NOT be implemented
//! following the recommendations in the NXP datasheet to set and read all the seven date and time registers in one go.
//!
//! TO DO: As the chip may be used for devices that are clocks only, without the calendar function
//! a convenient set_time() function could be added (sets only seconds, minutes and hours)

use super::{decode_bcd, encode_bcd, Error, Register, DEVICE_ADDRESS, PCF85063};
use embedded_hal_async::i2c::I2c;
use time::{Date, PrimitiveDateTime, Time};

impl<I2C, E> PCF85063<I2C>
where
    I2C: I2c<Error = E>,
{
    /// Read date and time all at once.
    pub async fn get_datetime(&mut self) -> Result<PrimitiveDateTime, Error<E>> {
        let mut data = [0; 7];
        self.i2c
            .write_read(DEVICE_ADDRESS, &[Register::SECONDS], &mut data)
            .await
            .map_err(Error::I2C)?;

        Ok(PrimitiveDateTime::new(
            Date::from_calendar_date(
                2000 + decode_bcd(data[6]) as i32,
                decode_bcd(data[5] & 0x1f).try_into()?,
                decode_bcd(data[3] & 0x3f),
            )?,
            Time::from_hms(
                decode_bcd(data[2] & 0x3f),
                decode_bcd(data[1] & 0b0111_1111),
                decode_bcd(data[0] & 0b0111_1111),
            )?,
        ))
    }

    /// Set date and time all at once.
    pub async fn set_datetime(&mut self, datetime: &PrimitiveDateTime) -> Result<(), Error<E>> {
        let payload = [
            Register::SECONDS, //first register
            encode_bcd(datetime.second()),
            encode_bcd(datetime.minute()),
            encode_bcd(datetime.hour()),
            encode_bcd(datetime.day()),
            encode_bcd(datetime.weekday().number_days_from_sunday()),
            encode_bcd(datetime.month().into()),
            encode_bcd((datetime.year() - 2000) as u8),
        ];
        self.i2c
            .write(DEVICE_ADDRESS, &payload)
            .await
            .map_err(Error::I2C)
    }

    /// Set only the time, date remains unchanged.
    ///
    /// Will return an 'Error::InvalidInputData' if any of the parameters is out of range.
    pub async fn set_time(&mut self, time: &Time) -> Result<(), Error<E>> {
        let payload = [
            Register::SECONDS, //first register
            encode_bcd(time.second()),
            encode_bcd(time.minute()),
            encode_bcd(time.hour()),
        ];
        self.i2c
            .write(DEVICE_ADDRESS, &payload)
            .await
            .map_err(Error::I2C)
    }
}
