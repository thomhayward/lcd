/// WIP!
/// - Currently only supports 4-bit mode.
/// - Some commands are not implemented.
/// 
/// Rust crate for using liquid crystal character displays based on the Hitachi HD44780 and
/// compatible chipsets.
/// 
/// This crate is a direct translation of the Arduino LiquidCrystal library found at
/// [https://github.com/arduino-libraries/LiquidCrystal].
///
mod lcd;
pub use lcd::LiquidCrystal;
