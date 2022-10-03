use crate::Time;

use super::{
    decode_bcd, encode_bcd, hal, BitFlags, Control, Error, Register, DEVICE_ADDRESS, PCF85063,
};
use hal::blocking::i2c::{Write, WriteRead};

impl<I2C, E> PCF85063<I2C>
where
    I2C: Write<Error = E> + WriteRead<Error = E>,
{
    /// Set the alarm minutes [0-59], keeping the AE bit unchanged.
    pub fn set_alarm_seconds(&mut self, seconds: u8) -> Result<(), Error<E>> {
        if seconds > 59 {
            return Err(Error::InvalidInputData);
        }
        let data: u8 = self.read_register(Register::SECOND_ALARM)?; // read current value
        let data: u8 = data & BitFlags::AE; // keep the AE bit as is
        let setting: u8 = encode_bcd(seconds);
        let data: u8 = data | setting;
        self.write_register(Register::SECOND_ALARM, data)
    }

    /// Set the alarm minutes [0-59], keeping the AE bit unchanged.
    pub fn set_alarm_minutes(&mut self, minutes: u8) -> Result<(), Error<E>> {
        if minutes > 59 {
            return Err(Error::InvalidInputData);
        }
        let data: u8 = self.read_register(Register::MINUTE_ALARM)?; // read current value
        let data: u8 = data & BitFlags::AE; // keep the AE bit as is
        let setting: u8 = encode_bcd(minutes);
        let data: u8 = data | setting;
        self.write_register(Register::MINUTE_ALARM, data)
    }

    /// Set the alarm hours [0-23], keeping the AE bit unchanged.
    pub fn set_alarm_hours(&mut self, hours: u8) -> Result<(), Error<E>> {
        if hours > 23 {
            return Err(Error::InvalidInputData);
        }
        let data: u8 = self.read_register(Register::HOUR_ALARM)?; // read current value
        let data: u8 = data & BitFlags::AE; // keep the AE bit as is
        let setting: u8 = encode_bcd(hours);
        let data: u8 = data | setting;
        self.write_register(Register::HOUR_ALARM, data)
    }

    /// Set the alarm day [1-31], keeping the AE bit unchanged.
    pub fn set_alarm_day(&mut self, day: u8) -> Result<(), Error<E>> {
        if !(1..=31).contains(&day) {
            return Err(Error::InvalidInputData);
        }
        let data: u8 = self.read_register(Register::DAY_ALARM)?; // read current value
        let data: u8 = data & BitFlags::AE; // keep the AE bit as is
        let setting: u8 = encode_bcd(day);
        let data: u8 = data | setting;
        self.write_register(Register::DAY_ALARM, data)
    }

    /// Set the alarm hours, minutes and seconds. Convenience function to call
    /// `Self::set_alarm_hours`, `Self::set_alarm_minutes`, and `Self::set_alarm_seconds`
    pub fn set_alarm_time(&mut self, time: &Time) -> Result<(), Error<E>> {
        self.set_alarm_hours(time.hours)?;
        self.set_alarm_minutes(time.minutes)?;
        self.set_alarm_seconds(time.seconds)
    }

    /// Set the alarm weekday [0-6], keeping the AE bit unchanged.
    pub fn set_alarm_weekday(&mut self, weekday: u8) -> Result<(), Error<E>> {
        if weekday > 6 {
            return Err(Error::InvalidInputData);
        }
        let data: u8 = self.read_register(Register::WEEKDAY_ALARM)?; // read current value
        let data: u8 = data & BitFlags::AE; // keep the AE bit as is
        let setting: u8 = encode_bcd(weekday);
        let data: u8 = data | setting;
        self.write_register(Register::WEEKDAY_ALARM, data)
    }

    /// Control alarm seconds (On: alarm enabled, Off: alarm disabled).
    pub fn control_alarm_seconds(&mut self, status: Control) -> Result<(), Error<E>> {
        match status {
            Control::Off => self.set_register_bit_flag(Register::SECOND_ALARM, BitFlags::AE),
            Control::On => self.clear_register_bit_flag(Register::SECOND_ALARM, BitFlags::AE),
        }
    }

    /// Control alarm minutes (On: alarm enabled, Off: alarm disabled).
    pub fn control_alarm_minutes(&mut self, status: Control) -> Result<(), Error<E>> {
        match status {
            Control::Off => self.set_register_bit_flag(Register::MINUTE_ALARM, BitFlags::AE),
            Control::On => self.clear_register_bit_flag(Register::MINUTE_ALARM, BitFlags::AE),
        }
    }

    /// Is alarm minutes enabled?
    pub fn is_alarm_minutes_enabled(&mut self) -> Result<bool, Error<E>> {
        let flag = self.is_register_bit_flag_high(Register::MINUTE_ALARM, BitFlags::AE)?;
        let flag = flag ^ true;
        Ok(flag)
    }

    /// Control alarm hours (On: alarm enabled, Off: alarm disabled).
    pub fn control_alarm_hours(&mut self, status: Control) -> Result<(), Error<E>> {
        match status {
            Control::Off => self.set_register_bit_flag(Register::HOUR_ALARM, BitFlags::AE),
            Control::On => self.clear_register_bit_flag(Register::HOUR_ALARM, BitFlags::AE),
        }
    }

    /// Is alarm hours enabled?
    pub fn is_alarm_hours_enabled(&mut self) -> Result<bool, Error<E>> {
        let flag = self.is_register_bit_flag_high(Register::HOUR_ALARM, BitFlags::AE)?;
        let flag = flag ^ true;
        Ok(flag)
    }

    /// Control alarm hours, minutes, and seconds in one go (On: alarm enabled, Off: alarm disabled).
    pub fn control_alarm_time(&mut self, status: Control) -> Result<(), Error<E>> {
        self.control_alarm_seconds(status)?;
        self.control_alarm_minutes(status)?;
        self.control_alarm_hours(status)
    }

    /// Control alarm day (On: alarm enabled, Off: alarm disabled).
    pub fn control_alarm_day(&mut self, status: Control) -> Result<(), Error<E>> {
        match status {
            Control::Off => self.set_register_bit_flag(Register::DAY_ALARM, BitFlags::AE),
            Control::On => self.clear_register_bit_flag(Register::DAY_ALARM, BitFlags::AE),
        }
    }

    /// Is alarm day enabled?
    pub fn is_alarm_day_enabled(&mut self) -> Result<bool, Error<E>> {
        let flag = self.is_register_bit_flag_high(Register::DAY_ALARM, BitFlags::AE)?;
        let flag = flag ^ true;
        Ok(flag)
    }

    /// Control alarm weekday (On: alarm enabled, Off: alarm disabled).
    pub fn control_alarm_weekday(&mut self, status: Control) -> Result<(), Error<E>> {
        match status {
            Control::Off => self.set_register_bit_flag(Register::WEEKDAY_ALARM, BitFlags::AE),
            Control::On => self.clear_register_bit_flag(Register::WEEKDAY_ALARM, BitFlags::AE),
        }
    }

    /// Is alarm weekday enabled?
    pub fn is_alarm_weekday_enabled(&mut self) -> Result<bool, Error<E>> {
        let flag = self.is_register_bit_flag_high(Register::WEEKDAY_ALARM, BitFlags::AE)?;
        let flag = flag ^ true;
        Ok(flag)
    }

    /// Enable or disable alarm interrupt.
    pub fn control_alarm_interrupt(&mut self, status: Control) -> Result<(), Error<E>> {
        match status {
            Control::On => self.set_register_bit_flag(Register::CONTROL_2, BitFlags::AIE),
            Control::Off => self.clear_register_bit_flag(Register::CONTROL_2, BitFlags::AIE),
        }
    }

    /// Read the alarm minutes setting.        
    pub fn get_alarm_minutes(&mut self) -> Result<u8, Error<E>> {
        let mut data = [0];
        self.i2c
            .write_read(DEVICE_ADDRESS, &[Register::MINUTE_ALARM], &mut data)
            .map_err(Error::I2C)?;
        Ok(decode_bcd(data[0]))
    }

    /// Read the alarm hours setting.
    pub fn get_alarm_hours(&mut self) -> Result<u8, Error<E>> {
        let mut data = [0];
        self.i2c
            .write_read(DEVICE_ADDRESS, &[Register::HOUR_ALARM], &mut data)
            .map_err(Error::I2C)?;
        Ok(decode_bcd(data[0]))
    }

    /// Read the alarm day setting.
    pub fn get_alarm_day(&mut self) -> Result<u8, Error<E>> {
        let mut data = [0];
        self.i2c
            .write_read(DEVICE_ADDRESS, &[Register::DAY_ALARM], &mut data)
            .map_err(Error::I2C)?;
        Ok(decode_bcd(data[0]))
    }

    /// Read the alarm weekday setting.
    pub fn get_alarm_weekday(&mut self) -> Result<u8, Error<E>> {
        let mut data = [0];
        self.i2c
            .write_read(DEVICE_ADDRESS, &[Register::WEEKDAY_ALARM], &mut data)
            .map_err(Error::I2C)?;
        Ok(decode_bcd(data[0]))
    }

    /// Get the alarm flag (if true, alarm event happened).
    pub fn get_alarm_flag(&mut self) -> Result<bool, Error<E>> {
        self.is_register_bit_flag_high(Register::CONTROL_2, BitFlags::AF)
    }

    /// Clear the alarm flag.
    pub fn clear_alarm_flag(&mut self) -> Result<(), Error<E>> {
        self.clear_register_bit_flag(Register::CONTROL_2, BitFlags::AF)
    }

    /// Check if alarm interrupt is enabled.
    pub fn is_alarm_interrupt_enabled(&mut self) -> Result<bool, Error<E>> {
        self.is_register_bit_flag_high(Register::CONTROL_2, BitFlags::AIE)
    }

    /// Shut off the alarms at once.
    pub fn disable_all_alarms(&mut self) -> Result<(), Error<E>> {
        self.control_alarm_minutes(Control::Off)?;
        self.control_alarm_hours(Control::Off)?;
        self.control_alarm_day(Control::Off)?;
        self.control_alarm_weekday(Control::Off)?;
        Ok(())
    }
}
