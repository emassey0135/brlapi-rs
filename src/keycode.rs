use binrw::{binrw, BinRead, BinWrite};
use bitflags::bitflags;
use bitfield_struct::bitfield;
use std::io::Cursor;
bitflags! {
  #[derive(Debug, PartialEq, Eq, Clone)]
  pub struct KeycodeFlags: u32 {
    const Shift = 1;
    const UpperCase = 1 << 1;
    const Control = 1 << 2;
    const Modifier1 = 1 << 3;
    const Meta = 1 << 3;
    const Modifier2 = 1 << 4;
    const AltGr = 1 << 4;
    const Modifier3 = 1 << 5;
    const Gui = 1 << 5;
    const Modifier4 = 1 << 6;
    const Escaped = 1 << 6;
    const Modifier5 = 1 << 7;
    const CapsLock = 1 << 7;
    const ToggleOn = 1 << 8;
    const Release = 1 << 8;
    const ToggleOff = 1 << 9;
    const Emulation0 = 1 << 9;
    const MotionRoute = 1 << 10;
    const Emulation1 = 1 << 10;
    const MotionScaled = 1 << 11;
    const MotionToLeft = 1 << 12;
  }
}
impl KeycodeFlags {
  const fn from_u32(value: u32) -> Self {
    KeycodeFlags::from_bits_truncate(value)
  }
  const fn into_u32(self) -> u32 {
    self.bits()
  }
}
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[repr(u8)]
pub enum KeycodeType {
  Keysym = 0,
  BrailleCommand = 1,
}
impl KeycodeType {
  const fn from_u8(value: u8) -> Self {
    match value {
      0 => KeycodeType::Keysym,
      _ => KeycodeType::BrailleCommand,
    }
  }
  const fn into_u8(self) -> u8 {
    self as _
  }
}
#[bitfield(u64)]
#[derive(PartialEq, Eq)]
pub struct Keycode {
  #[bits(29)]
  code: u32,
  #[bits(3, from = KeycodeType::from_u8, into = KeycodeType::into_u8)]
  type_flags: KeycodeType,
  #[bits(32, from = KeycodeFlags::from_u32, into = KeycodeFlags::into_u32)]
  flags: KeycodeFlags,
}
#[binrw]
#[brw(big)]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum BrailleCommand {
  #[brw(magic(0u32))]
  NoOp,
  #[brw(magic(1u32))]
  LineUp,
  #[brw(magic(2u32))]
  LineDown,
  #[brw(magic(3u32))]
  SeveralLinesUp,
  #[brw(magic(4u32))]
  SeveralLinesDown,
  #[brw(magic(5u32))]
  PreviousDifferentLine,
  #[brw(magic(6u32))]
  NextDifferentLine,
  #[brw(magic(7u32))]
  PreviousDifferentAttributesLine,
  #[brw(magic(8u32))]
  NextDifferentAttributesLine,
  #[brw(magic(9u32))]
  Top,
  #[brw(magic(10u32))]
  Bottom,
  #[brw(magic(11u32))]
  TopLeft,
  #[brw(magic(12u32))]
  BottomLeft,
}
impl BrailleCommand {
  pub fn from_u32(value: u32) -> Self {
    let bytes = value.to_be_bytes();
    let mut stream = Cursor::new(bytes);
    BrailleCommand::read(&mut stream).unwrap()
  }
  pub fn into_u32(self) -> u32 {
    let mut stream = Cursor::new(Vec::new());
    self.write(&mut stream).unwrap();
    let bytes = stream.into_inner();
    u32::from_be_bytes(bytes.try_into().unwrap())
  }
}
