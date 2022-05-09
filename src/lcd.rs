use embedded_hal::digital::v2::{OutputPin, PinState};
use embedded_hal::blocking::delay::DelayUs;

const LCD_CLEARDISPLAY: u8 = 0x01;
const LCD_RETURNHOME: u8 = 0x02;
const LCD_ENTRYMODESET: u8 = 0x04;
const LCD_DISPLAYCONTROL: u8 = 0x08;
// const LCD_CURSORSHIFT: u8 = 0x10;
const LCD_FUNCTIONSET: u8 = 0x20;
// const LCD_SETCGRAMADDR: u8 = 0x40;
const LCD_SETDDRAMADDR: u8 = 0x80;

// flags for display entry mode
// const LCD_ENTRYRIGHT: u8 = 0x00;
const LCD_ENTRYLEFT: u8 = 0x02;
// const LCD_ENTRYSHIFTINCREMENT: u8 = 0x01;
const LCD_ENTRYSHIFTDECREMENT: u8 = 0x00;

// flags for display on/off control
const LCD_DISPLAYON: u8 = 0x04;
const LCD_CURSORON: u8 = 0x02;
const LCD_BLINKON: u8 = 0x01;

// const LCD_8BITMODE: u8 = 0x10;
const LCD_4BITMODE: u8 = 0x00;
const LCD_2LINE: u8 = 0x08;
const LCD_1LINE: u8 = 0x00;
const LCD_5X10DOTS: u8 = 0x04;
const LCD_5X8DOTS: u8 = 0x00;

pub struct LiquidCrystal<RS, EN, D4, D5, D6, D7, DELAY, ERROR>
where
    RS: OutputPin<Error = ERROR>,
    EN: OutputPin<Error = ERROR>,
    D4: OutputPin<Error = ERROR>,
    D5: OutputPin<Error = ERROR>,
    D6: OutputPin<Error = ERROR>,
    D7: OutputPin<Error = ERROR>,
    DELAY: DelayUs<u16>,
{
    rs: RS,
    en: EN,
    d4: D4,
    d5: D5,
    d6: D6,
    d7: D7,
    columns: u8,
    lines: u8,
    function: u8,
    control: u8,
    mode: u8,
    row_offsets: [u8; 4],
    delay: DELAY,
}

impl<RS, EN, D4, D5, D6, D7, DELAY, ERROR> LiquidCrystal<RS, EN, D4, D5, D6, D7, DELAY, ERROR>
where
    RS: OutputPin<Error = ERROR>,
    EN: OutputPin<Error = ERROR>,
    D4: OutputPin<Error = ERROR>,
    D5: OutputPin<Error = ERROR>,
    D6: OutputPin<Error = ERROR>,
    D7: OutputPin<Error = ERROR>,
    DELAY: DelayUs<u16>,
{
    /// Creates a new LiquidCrystal struct, but does not initialise the display!
    /// ```
    /// let rs = gpio.gpio13.into_push_pull_output();
    /// let en = gpio.gpio12.into_push_pull_output();
    /// let d4 = gpio.gpio14.into_push_pull_output();
    /// let d5 = gpio.gpio27.into_push_pull_output();
    /// let d6 = gpio.gpio26.into_push_pull_output();
    /// let d7 = gpio.gpio25.into_push_pull_output();
    /// let delay = esp32_hal::delay::Delay::new();
    /// let lcd = LiquidCrystal::new(rs, en, d4, d5, d6, d7, delay);
    /// // initialise the display with number of columns and rows
    /// lcd.begin(16, 4, 0)?;
    /// ```
    pub fn new(rs: RS, en: EN, d4: D4, d5: D5, d6: D6, d7: D7, delay: DELAY) -> Self {
        Self {
            rs,
            en,
            d4,
            d5,
            d6,
            d7,
            columns: 0,
            lines: 0,
            function: LCD_4BITMODE | LCD_1LINE | LCD_5X8DOTS,
            control: 0,
            mode: 0,
            row_offsets: [0; 4],
            delay,
        }
    }
    //
    pub fn begin(&mut self, columns: u8, lines: u8, dotsize: u8) -> Result<&Self, ERROR> {
        self.columns = columns;
        self.lines = lines;

        if lines > 1 {
            self.function |= LCD_2LINE;
        }

        self.row_offsets[0] = 0x00;
        self.row_offsets[1] = 0x40;
        self.row_offsets[2] = columns; // + 0x00
        self.row_offsets[3] = columns + 0x40;

        // for some 1 line displays you can select a 10 pixel high font
        if (dotsize != LCD_5X8DOTS) && (lines == 1) {
            self.function |= LCD_5X10DOTS;
        }

        // initialise, settling is probably unneccessary
        self.delay.delay_us(50000);
        self.rs.set_low()?;
        self.en.set_low()?;

        // put display into 4-bit mode
        self.write_4bits(0x03)?;
        self.delay.delay_us(4500);
        self.write_4bits(0x03)?;
        self.delay.delay_us(4500);
        self.write_4bits(0x03)?;
        self.delay.delay_us(150);
        self.write_4bits(0x02)?;

        // set # lines, font size, etc.
        self.command(LCD_FUNCTIONSET | self.function)?;
        self.display(true)?;
        self.clear()?;

        self.mode = LCD_ENTRYLEFT | LCD_ENTRYSHIFTDECREMENT;
        self.command(LCD_ENTRYMODESET | self.mode)
    }
    /// Enables or disables the display.
    ///
    pub fn display(&mut self, on: bool) -> Result<&Self, ERROR> {
        match on {
            true => self.control |= LCD_DISPLAYON,
            false => self.control &= !LCD_DISPLAYON,
        }
        self.command(LCD_DISPLAYCONTROL | self.control)
    }
    /// Enable or disable the underline cursor.
    ///
    pub fn cursor(&mut self, on: bool) -> Result<&Self, ERROR> {
        match on {
            true => self.control |= LCD_CURSORON,
            false => self.control &= !LCD_CURSORON,
        }
        self.command(LCD_DISPLAYCONTROL | self.control)
    }
    /// Enable or disable the blinking cursor.
    ///
    pub fn blink(&mut self, on: bool) -> Result<&Self, ERROR> {
        match on {
            true => self.control |= LCD_BLINKON,
            false => self.control &= !LCD_BLINKON,
        }
        self.command(LCD_DISPLAYCONTROL | self.control)
    }
    /// Clears the display.
    ///
    pub fn clear(&mut self) -> Result<&Self, ERROR> {
        self.command(LCD_CLEARDISPLAY)?;
        self.delay.delay_us(2000);
        Ok(self)
    }
    /// Returns the cursor to the home position.
    ///
    pub fn home(&mut self) -> Result<&Self, ERROR> {
        self.command(LCD_RETURNHOME)?;
        self.delay.delay_us(2000);
        Ok(self)
    }
    /// Sets the cursor position. `column` and `row` are zero indexed.
    ///
    pub fn set_cursor(&mut self, column: u8, mut row: u8) -> Result<&Self, ERROR> {
        let max_lines = self.row_offsets.len() as u8;
        if row >= max_lines {
            row = max_lines - 1;
        }
        if row >= self.lines {
            row = self.lines - 1;
        }
        let addr: u8 = column + self.row_offsets[row as usize];
        self.command(LCD_SETDDRAMADDR | addr)
    }
    //
    pub fn command(&mut self, value: u8) -> Result<&Self, ERROR> {
        self.send(value, PinState::Low)
    }
    pub fn write(&mut self, value: u8) -> Result<&Self, ERROR> {
        self.send(value, PinState::High)
    }
    /// Low level command, for writing data to the display.
    ///
    fn send(&mut self, value: u8, mode: PinState) -> Result<&Self, ERROR> {
        self.rs.set_state(mode)?;
        self.write_4bits(value >> 4)?;
        self.write_4bits(value)
    }
    //
    fn write_4bits(&mut self, value: u8) -> Result<&Self, ERROR> {
        for i in 0..4 {
            let state = (value >> i) & 0x01  == 0x01;
            match i {
                0 => self.d4.set_state(state.into())?,
                1 => self.d5.set_state(state.into())?,
                2 => self.d6.set_state(state.into())?,
                3 => self.d7.set_state(state.into())?,
                _ => unreachable!()
            }
        }
        self.pulse()
    }
    //
    fn pulse(&mut self) -> Result<&Self, ERROR> {
        self.en.set_low()?;
        self.delay.delay_us(1);
        self.en.set_high()?;
        self.delay.delay_us(1);
        self.en.set_low()?;
        self.delay.delay_us(100);
        Ok(self)
    }
}
