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
  #[brw(magic(13u32))]
  PreviousParagraph,
  #[brw(magic(14u32))]
  NextParagraph,
  #[brw(magic(15u32))]
  PreviousPrompt,
  #[brw(magic(16u32))]
  NextPrompt,
  #[brw(magic(17u32))]
  SearchBackwards,
  #[brw(magic(18u32))]
  SearchForwards,
  #[brw(magic(19u32))]
  PreviousCharacter,
  #[brw(magic(20u32))]
  NextCharacter,
  #[brw(magic(21u32))]
  PreviousHalfWindow,
  #[brw(magic(22u32))]
  NextHalfWindow,
  #[brw(magic(23u32))]
  PreviousFullWindow,
  #[brw(magic(24u32))]
  NextFullWindow,
  #[brw(magic(25u32))]
  PreviousFullWindowSkippingBlanks,
  #[brw(magic(26u32))]
  NextFullWindowSkippingBlanks,
  #[brw(magic(27u32))]
  LineBeginning,
  #[brw(magic(28u32))]
  LineEnd,
  #[brw(magic(29u32))]
  Home,
  #[brw(magic(30u32))]
  Back,
  #[brw(magic(31u32))]
  Return,
  #[brw(magic(32u32))]
  ToggleFrozen,
  #[brw(magic(33u32))]
  ToggleDisplayMode,
  #[brw(magic(34u32))]
  Toggle8Dot,
  #[brw(magic(35u32))]
  ToggleSlidingWindow,
  #[brw(magic(36u32))]
  ToggleSkipIdenticalLines,
  #[brw(magic(37u32))]
  ToggleSkipBlankWindows,
  #[brw(magic(38u32))]
  ToggleScreenCursorVisibility,
  #[brw(magic(39u32))]
  ToggleHideScreenCursor,
  #[brw(magic(40u32))]
  ToggleTrackScreenCursor,
  #[brw(magic(41u32))]
  ToggleScreenCursorStyle,
  #[brw(magic(42u32))]
  ToggleScreenCursorBlink,
  #[brw(magic(43u32))]
  ToggleAttributeUnderline,
  #[brw(magic(44u32))]
  ToggleAttributeBlink,
  #[brw(magic(45u32))]
  ToggleCapitalBlink,
  #[brw(magic(46u32))]
  ToggleAlertTunes,
  #[brw(magic(47u32))]
  ToggleAutoRepeat,
  #[brw(magic(48u32))]
  ToggleAutoSpeak,
  #[brw(magic(49u32))]
  Help,
  #[brw(magic(50u32))]
  Info,
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
