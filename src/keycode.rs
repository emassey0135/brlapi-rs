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
  #[brw(magic(51u32))]
  LearnMode,
  #[brw(magic(52u32))]
  PreferencesMenu,
  #[brw(magic(53u32))]
  SavePreferences,
  #[brw(magic(54u32))]
  LoadPreferences,
  #[brw(magic(55u32))]
  MenuFirstItem,
  #[brw(magic(56u32))]
  MenuLastItem,
  #[brw(magic(57u32))]
  MenuPreviousItem,
  #[brw(magic(58u32))]
  MenuNextItem,
  #[brw(magic(59u32))]
  MenuPreviousSetting,
  #[brw(magic(60u32))]
  MenuNextSetting,
  #[brw(magic(61u32))]
  Mute,
  #[brw(magic(62u32))]
  SpeechHome,
  #[brw(magic(63u32))]
  SayLine,
  #[brw(magic(64u32))]
  SayFromTop,
  #[brw(magic(65u32))]
  SayToBottom,
  #[brw(magic(66u32))]
  DecreaseSpeechRate,
  #[brw(magic(67u32))]
  IncreaseSpeechRate,
  #[brw(magic(68u32))]
  DecreaseSpeechVolume,
  #[brw(magic(69u32))]
  IncreaseSpeechVolume,
  #[brw(magic(70u32))]
  PreviousVirtualTerminal,
  #[brw(magic(71u32))]
  NextVirtualTerminal,
  #[brw(magic(72u32))]
  ScreenCursorToCurrentLine,
  #[brw(magic(73u32))]
  Paste,
  #[brw(magic(74u32))]
  RestartBrailleDriver,
  #[brw(magic(75u32))]
  RestartSpeechDriver,
  #[brw(magic(76u32))]
  BrailleDisplayOffline,
  #[brw(magic(77u32))]
  CycleShift,
  #[brw(magic(78u32))]
  CycleUpper,
  #[brw(magic(79u32))]
  CycleControl,
  #[brw(magic(80u32))]
  CycleMeta,
  #[brw(magic(81u32))]
  ShowTime,
  #[brw(magic(82u32))]
  PreviousMenuLevel,
  #[brw(magic(83u32))]
  ToggleAutoSpeakSelectedLine,
  #[brw(magic(84u32))]
  ToggleAutoSpeakSelectedCharacter,
  #[brw(magic(85u32))]
  ToggleAutoSpeakInsertedCharacters,
  #[brw(magic(86u32))]
  ToggleAutoSpeakDeletedCharacters,
  #[brw(magic(87u32))]
  ToggleAutoSpeakReplacedCharacters,
  #[brw(magic(88u32))]
  ToggleAutoSpeakCompletedWords,
  #[brw(magic(89u32))]
  SpeakCurrentCharacter,
  #[brw(magic(90u32))]
  SpeakPreviousCharacter,
  #[brw(magic(91u32))]
  SpeakNextCharacter,
  #[brw(magic(92u32))]
  SpeakCurrentWord,
  #[brw(magic(93u32))]
  SpeakPreviousWord,
  #[brw(magic(94u32))]
  SpeakNextWord,
  #[brw(magic(95u32))]
  SpeakCurrentLine,
  #[brw(magic(96u32))]
  SpeakPreviousLine,
  #[brw(magic(97u32))]
  SpeakNextLine,
  #[brw(magic(98u32))]
  SpeakFirstCharacter,
  #[brw(magic(99u32))]
  SpeakLastCharacter,
  #[brw(magic(100u32))]
  SpeakFirstLine,
  #[brw(magic(101u32))]
  SpeakLastLine,
  #[brw(magic(102u32))]
  DescribeCurrentCharacter,
  #[brw(magic(103u32))]
  SpellCurrentWord,
  #[brw(magic(104u32))]
  ScreenCursorToSpeechCursor,
  #[brw(magic(105u32))]
  SpeakSpeechCursorLocation,
  #[brw(magic(106u32))]
  ToggleSpeechCursorVisibility,
  #[brw(magic(107u32))]
  SaveClipboard,
  #[brw(magic(108u32))]
  RestoreClipboard,
  #[brw(magic(109u32))]
  ToggleUnicodeBrailleInput,
  #[brw(magic(110u32))]
  ToggleBrailleKeyboard,
  #[brw(magic(111u32))]
  ClearModifiers,
  #[brw(magic(112u32))]
  CycleAltGr,
  #[brw(magic(113u32))]
  CycleGui,
  #[brw(magic(114u32))]
  StopBrailleDriver,
  #[brw(magic(115u32))]
  StartBrailleDriver,
  #[brw(magic(116u32))]
  StopSpeechDriver,
  #[brw(magic(117u32))]
  StartSpeechDriver,
  #[brw(magic(118u32))]
  StopScreenDriver,
  #[brw(magic(119u32))]
  StartScreenDriver,
  #[brw(magic(120u32))]
  SelectPreviousVirtualTerminal,
  #[brw(magic(121u32))]
  SelectNextVirtualTerminal,
  #[brw(magic(122u32))]
  PreviousNonBlankWindow,
  #[brw(magic(123u32))]
  NextNonBlankWindow,
  #[brw(magic(124u32))]
  ToggleTouchNavigation,
  #[brw(magic(125u32))]
  SpeakIndent,
  #[brw(magic(126u32))]
  ToggleAutoSpeakIndent,
  #[brw(magic(127u32))]
  Refresh,
  #[brw(magic(128u32))]
  ShowStatusIndicators,
  #[brw(magic(129u32))]
  ClearTextSelection,
  #[brw(magic(130u32))]
  SelectAll,
  #[brw(magic(131u32))]
  HostCopy,
  #[brw(magic(132u32))]
  HostCut,
  #[brw(magic(133u32))]
  HostPaste,
  #[brw(magic(134u32))]
  GuiTitle,
  #[brw(magic(135u32))]
  GuiBrailleActions,
  #[brw(magic(136u32))]
  GuiHome,
  #[brw(magic(137u32))]
  GuiBack,
  #[brw(magic(138u32))]
  GuiDeviceSettings,
  #[brw(magic(139u32))]
  GuiDeviceOptions,
  #[brw(magic(140u32))]
  GuiApplicationsList,
  #[brw(magic(141u32))]
  GuiApplicationMenu,
  #[brw(magic(142u32))]
  GuiApplicationAlerts,
  #[brw(magic(143u32))]
  GuiActiveArea,
  #[brw(magic(144u32))]
  GuiPreviousArea,
  #[brw(magic(145u32))]
  GuiNextArea,
  #[brw(magic(146u32))]
  GuiFirstItem,
  #[brw(magic(147u32))]
  GuiPreviousItem,
  #[brw(magic(148u32))]
  GuiNextItem,
  #[brw(magic(149u32))]
  GuiLastItem,
  #[brw(magic(150u32))]
  DecreaseSpeechPitch,
  #[brw(magic(151u32))]
  IncreaseSpeechPitch,
  #[brw(magic(152u32))]
  SayAll,
  #[brw(magic(153u32))]
  ToggleContracted,
  #[brw(magic(154u32))]
  Toggle6DotComputerBraille,
  #[brw(magic(155u32))]
  ResetPreferences,
  #[brw(magic(156u32))]
  ToggleAutoSpeakEmptyLines,
  #[brw(magic(1u16))]
  RouteCursorToCharacter { column: u16 },
  #[brw(magic(2u16))]
  NewClipboardAtCharacter { column: u16 },
  #[brw(magic(3u16))]
  AppendToCharacter { column: u16 },
  #[brw(magic(4u16))]
  RectangularCopyToCharacter { column: u16 },
  #[brw(magic(5u16))]
  LinearCopyToCharacter { column: u16 },
  #[brw(magic(6u16))]
  SwitchToVirtualTerminal { terminal: u16 },
  #[brw(magic(7u16))]
  PreviousIndent { column: u16 },
  #[brw(magic(8u16))]
  NextIndent { column: u16 },
  #[brw(magic(9u16))]
  DescribeCharacter { column: u16 },
  #[brw(magic(10u16))]
  SetWindowLeft { column: u16 },
  #[brw(magic(11u16))]
  SetMark { marker_number: u16 },
  #[brw(magic(12u16))]
  GoToMark { marker_number: u16 },
  #[brw(magic(13u16))]
  GoToLine { row: u16 },
  #[brw(magic(14u16))]
  PreviousLineWithDifferentCharacter { column: u16 },
  #[brw(magic(15u16))]
  NextLineWithDifferentCharacter { column: u16 },
  #[brw(magic(16u16))]
  Copy { start_column: u8, end_column: u8 },
  #[brw(magic(17u16))]
  Append { start_column: u8, end_column: u8 },
  #[brw(magic(18u16))]
  PasteFromHistory { history_entry: u16 },
  #[brw(magic(19u16))]
  SetTextTable { index: u16 },
  #[brw(magic(20u16))]
  SetAttributesTable { index: u16 },
  #[brw(magic(21u16))]
  SetContractionTable { index: u16 },
  #[brw(magic(22u16))]
  SetKeyboardTable { index: u16 },
  #[brw(magic(23u16))]
  SetLanguageProfile { index: u16 },
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
