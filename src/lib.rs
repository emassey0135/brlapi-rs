#![deny(clippy::all)]
#![warn(clippy::pedantic)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::ref_option)]
#![allow(clippy::trivially_copy_pass_by_ref)]
#![allow(clippy::match_same_arms)]
pub mod keycode;
use crate::keycode::Keycode;
use binrw::{NullString, binrw};
use bitflags::bitflags;
#[binrw]
#[brw(big, repr(u32))]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ErrorCode {
  Success = 0,
  NotEnoughMemory = 1,
  TtyBusy = 2,
  DeviceBusy = 3,
  UnknownInstruction = 4,
  IllegalInstruction = 5,
  InvalidParameter = 6,
  InvalidPacket = 7,
  ConnectionRefused = 8,
  OperationNotSupported = 9,
  GetaddrinfoError = 10,
  LibcError = 11,
  UnknownTty = 12,
  BadProtocolVersion = 13,
  UnexpectedEof = 14,
  EmptyKeyFile = 15,
  DriverPacketTooLarge = 16,
  AuthenticationFailed = 17,
  ReadOnlyParameter = 18,
}
#[binrw]
#[brw(big)]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum AuthType {
  #[brw(magic(b"\0\0\0N"))]
  None,
  #[brw(magic(b"\0\0\0K"))]
  Key,
  #[brw(magic(b"\0\0\0C"))]
  Credentials,
}
bitflags! {
  #[derive(Debug, PartialEq, Eq, Clone)]
  pub struct WriteFlags: u32 {
    const DisplayNumber = 1;
    const Region = 1 << 1;
    const Text = 1 << 2;
    const And = 1 << 3;
    const Or = 1 << 4;
    const Cursor = 1 << 5;
    const Charset = 1 << 6;
  }
}
bitflags! {
  #[derive(Debug, PartialEq, Eq, Clone)]
  pub struct ParameterRequestFlags: u32 {
    const Global = 1;
    const IncludeSelf = 1 << 1;
    const Get = 1 << 8;
    const Subscribe = 1 << 9;
    const Unsubscribe = 1 << 10;
  }
}
bitflags! {
  #[derive(Debug, PartialEq, Eq, Clone)]
  pub struct ParameterValueFlags: u32 {
    const Global = 1;
  }
}
#[binrw]
#[brw(big)]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum PacketType {
  #[brw(magic(b"\0\0\0A"))]
  Ack,
  #[brw(magic(b"\0\0\0e"))]
  Error,
  #[brw(magic(b"\0\0\0E"))]
  Exception,
  #[brw(magic(b"\0\0\0k"))]
  Key,
  #[brw(magic(b"\0\0\0w"))]
  Write,
  #[brw(magic(b"\0\0\0p"))]
  Packet,
  #[brw(magic(b"\0\0\0v"))]
  Version,
  #[brw(magic(b"\0\0\0a"))]
  Auth,
  #[brw(magic(b"\0\0\0n"))]
  GetDriverName,
  #[brw(magic(b"\0\0\0d"))]
  GetModelId,
  #[brw(magic(b"\0\0\0s"))]
  GetDisplaySize,
  #[brw(magic(b"\0\0\0t"))]
  EnterTtyMode,
  #[brw(magic(b"\0\0\0F"))]
  SetFocus,
  #[brw(magic(b"\0\0\0L"))]
  LeaveTtyMode,
  #[brw(magic(b"\0\0\0m"))]
  IgnoreKeyRanges,
  #[brw(magic(b"\0\0\0u"))]
  AcceptKeyRanges,
  #[brw(magic(b"\0\0\0*"))]
  EnterRawMode,
  #[brw(magic(b"\0\0\0#"))]
  LeaveRawMode,
  #[brw(magic(b"\0\0\0S"))]
  SuspendDriver,
  #[brw(magic(b"\0\0\0R"))]
  ResumeDriver,
  #[brw(magic(b"\0\0\0Z"))]
  Synchronize,
  #[brw(magic(b"\0\0PR"))]
  ParameterRequest,
  #[brw(magic(b"\0\0PV"))]
  ParameterValue,
  #[brw(magic(b"\0\0PU"))]
  ParameterUpdate,
}
fn calculate_write_flags(
  display_number: &Option<u32>,
  region: &Option<(u32, u32)>,
  text: &Option<Vec<u8>>,
  and: &Option<Vec<u8>>,
  or: &Option<Vec<u8>>,
  cursor: &Option<u32>,
  charset: &Option<Vec<u8>>,
) -> WriteFlags {
  let mut flags = WriteFlags::empty();
  if display_number.is_some() {
    flags |= WriteFlags::DisplayNumber;
  }
  if region.is_some() {
    flags |= WriteFlags::Region;
  }
  if text.is_some() {
    flags |= WriteFlags::Text;
  }
  if and.is_some() {
    flags |= WriteFlags::And;
  }
  if or.is_some() {
    flags |= WriteFlags::Or;
  }
  if cursor.is_some() {
    flags |= WriteFlags::Cursor;
  }
  if charset.is_some() {
    flags |= WriteFlags::Charset;
  }
  flags
}
#[binrw]
#[brw(big)]
#[br(import(size: u32, ty: PacketType))]
#[br(assert(size as usize == self.size()))]
#[derive(Debug, PartialEq, Eq, Clone)]
#[allow(clippy::cast_possible_truncation)]
pub enum ClientPacketData {
  #[br(pre_assert(ty == PacketType::Write))]
  Write {
    #[br(map(|bits: u32| WriteFlags::from_bits_truncate(bits)))]
    #[bw(map(|flags| flags.bits()))]
    #[br(temp)]
    #[bw(calc(calculate_write_flags(display_number, region, text, and, or, cursor, charset)))]
    flags: WriteFlags,
    #[br(if(flags.contains(WriteFlags::DisplayNumber)))]
    #[bw(if(WriteFlags::from_bits_truncate(flags).contains(WriteFlags::DisplayNumber)))]
    display_number: Option<u32>,
    #[br(if(flags.contains(WriteFlags::Region)))]
    #[bw(if(WriteFlags::from_bits_truncate(flags).contains(WriteFlags::Region)))]
    region: Option<(u32, u32)>,
    #[br(if(flags.contains(WriteFlags::Text)))]
    #[bw(if(WriteFlags::from_bits_truncate(flags).contains(WriteFlags::Text)))]
    #[br(temp)]
    #[bw(calc(text.as_ref().unwrap().len() as u32))]
    text_len: u32,
    #[br(if(flags.contains(WriteFlags::Text)))]
    #[bw(if(WriteFlags::from_bits_truncate(flags).contains(WriteFlags::Text)))]
    #[br(count(text_len))]
    text: Option<Vec<u8>>,
    #[br(if(flags.contains(WriteFlags::And)))]
    #[bw(if(WriteFlags::from_bits_truncate(flags).contains(WriteFlags::And)))]
    #[br(count(region.unwrap().1))]
    and: Option<Vec<u8>>,
    #[br(if(flags.contains(WriteFlags::Or)))]
    #[bw(if(WriteFlags::from_bits_truncate(flags).contains(WriteFlags::Or)))]
    #[br(count(region.unwrap().1))]
    or: Option<Vec<u8>>,
    #[br(if(flags.contains(WriteFlags::Cursor)))]
    #[bw(if(WriteFlags::from_bits_truncate(flags).contains(WriteFlags::Cursor)))]
    cursor: Option<u32>,
    #[br(if(flags.contains(WriteFlags::Charset)))]
    #[bw(if(WriteFlags::from_bits_truncate(flags).contains(WriteFlags::Charset)))]
    #[br(temp)]
    #[bw(calc(charset.as_ref().unwrap().len() as u32))]
    charset_len: u32,
    #[br(if(flags.contains(WriteFlags::Charset)))]
    #[bw(if(WriteFlags::from_bits_truncate(flags).contains(WriteFlags::Charset)))]
    #[br(count(charset_len))]
    charset: Option<Vec<u8>>,
  },
  #[br(pre_assert(ty == PacketType::Packet))]
  Packet {
    #[br(count(size))]
    packet: Vec<u8>,
  },
  #[br(pre_assert(ty == PacketType::Version))]
  Version { version: u32 },
  #[br(pre_assert(ty == PacketType::Auth))]
  #[br(assert(auth_type == AuthType::Key))]
  #[bw(assert(*auth_type == AuthType::Key))]
  Auth {
    auth_type: AuthType,
    key: NullString,
  },
  #[br(pre_assert(ty == PacketType::GetDriverName))]
  GetDriverName,
  #[br(pre_assert(ty == PacketType::GetModelId))]
  GetModelId,
  #[br(pre_assert(ty == PacketType::GetDisplaySize))]
  GetDisplaySize,
  #[br(pre_assert(ty == PacketType::EnterTtyMode))]
  EnterTtyMode {
    #[br(temp)]
    #[bw(calc(ttys.len() as u32))]
    ttys_len: u32,
    #[br(count(ttys_len))]
    ttys: Vec<u32>,
    #[br(temp)]
    #[bw(calc(driver.len() as u32))]
    driver_len: u32,
    #[br(count(driver_len))]
    driver: Vec<u8>,
  },
  #[br(pre_assert(ty == PacketType::SetFocus))]
  SetFocus { tty: u32 },
  #[br(pre_assert(ty == PacketType::LeaveTtyMode))]
  LeaveTtyMode,
  #[br(pre_assert(ty == PacketType::IgnoreKeyRanges))]
  IgnoreKeyRanges {
    #[br(count(size/16))]
    ranges: Vec<(u64, u64)>,
  },
  #[br(pre_assert(ty == PacketType::AcceptKeyRanges))]
  AcceptKeyRanges {
    #[br(count(size/16))]
    ranges: Vec<(u64, u64)>,
  },
  #[br(pre_assert(ty == PacketType::EnterRawMode))]
  #[brw(magic(0xdead_beefu64))]
  EnterRawMode {
    #[br(temp)]
    #[bw(calc(driver.len() as u8))]
    driver_len: u8,
    #[br(count(driver_len))]
    driver: Vec<u8>,
  },
  #[br(pre_assert(ty == PacketType::LeaveRawMode))]
  LeaveRawMode,
  #[br(pre_assert(ty == PacketType::SuspendDriver))]
  #[brw(magic(0xdead_beefu64))]
  SuspendDriver {
    #[br(temp)]
    #[bw(calc(driver.len() as u8))]
    driver_len: u8,
    #[br(count(driver_len))]
    driver: Vec<u8>,
  },
  #[br(pre_assert(ty == PacketType::ResumeDriver))]
  ResumeDriver,
  #[br(pre_assert(ty == PacketType::Synchronize))]
  Synchronize,
  #[br(pre_assert(ty == PacketType::ParameterRequest))]
  ParameterRequest {
    #[br(map(|bits: u32| ParameterRequestFlags::from_bits_truncate(bits)))]
    #[bw(map(ParameterRequestFlags::bits))]
    flags: ParameterRequestFlags,
    parameter: u32,
    sub_parameter: u64,
  },
  #[br(pre_assert(ty == PacketType::ParameterValue))]
  ParameterValue {
    #[br(map(|bits: u32| ParameterValueFlags::from_bits_truncate(bits)))]
    #[bw(map(ParameterValueFlags::bits))]
    flags: ParameterValueFlags,
    parameter: u32,
    sub_parameter: u64,
    #[br(count(size-16))]
    value: Vec<u8>,
  },
}
impl ClientPacketData {
  fn size(&self) -> usize {
    match self {
      ClientPacketData::Version { version: _ } => 4,
      ClientPacketData::Auth { auth_type: _, key } => key.len() + 4,
      ClientPacketData::GetDriverName => 0,
      ClientPacketData::GetModelId => 0,
      ClientPacketData::GetDisplaySize => 0,
      ClientPacketData::EnterTtyMode { ttys, driver } => 8 + ttys.len() + driver.len(),
      ClientPacketData::SetFocus { tty: _ } => 4,
      ClientPacketData::LeaveTtyMode => 0,
      ClientPacketData::IgnoreKeyRanges { ranges } => ranges.len() * 16,
      ClientPacketData::AcceptKeyRanges { ranges } => ranges.len() * 16,
      ClientPacketData::Write {
        display_number,
        region,
        text,
        and,
        or,
        cursor,
        charset,
      } => {
        let mut size = 4;
        if display_number.is_some() {
          size += 4;
        }
        if region.is_some() {
          size += 8;
        }
        if let Some(text) = text {
          size += text.len();
        }
        if let Some(and) = and {
          size += and.len();
        }
        if let Some(or) = or {
          size += or.len();
        }
        if cursor.is_some() {
          size += 4;
        }
        if let Some(charset) = charset {
          size += charset.len();
        }
        size
      }
      ClientPacketData::EnterRawMode { driver } => 8 + driver.len(),
      ClientPacketData::LeaveRawMode => 0,
      ClientPacketData::Packet { packet } => packet.len(),
      ClientPacketData::SuspendDriver { driver } => 8 + driver.len(),
      ClientPacketData::ResumeDriver => 0,
      ClientPacketData::Synchronize => 0,
      ClientPacketData::ParameterRequest {
        flags: _,
        parameter: _,
        sub_parameter: _,
      } => 16,
      ClientPacketData::ParameterValue {
        flags: _,
        parameter: _,
        sub_parameter: _,
        value,
      } => 16 + value.len(),
    }
  }
}
#[binrw]
#[brw(big)]
#[br(import(size: u32, ty: PacketType))]
#[br(assert(size as usize == self.size()))]
#[derive(Debug, PartialEq, Eq, Clone)]
#[allow(clippy::cast_possible_truncation)]
pub enum ServerPacketData {
  #[br(pre_assert(ty == PacketType::Ack))]
  Ack,
  #[br(pre_assert(ty == PacketType::Error))]
  Error {
    code: ErrorCode,
    packet_type: PacketType,
    #[br(count(size-8))]
    packet_data: Vec<u8>,
  },
  #[br(pre_assert(ty == PacketType::Exception))]
  Exception {
    #[br(count(size))]
    packet: Vec<u8>,
  },
  #[br(pre_assert(ty == PacketType::Key))]
  Key {
    #[br(map(|bits: u64| bits.into()))]
    #[bw(map(|code| u64::from(*code)))]
    key: Keycode,
  },
  #[br(pre_assert(ty == PacketType::Packet))]
  Packet {
    #[br(count(size))]
    packet: Vec<u8>,
  },
  #[br(pre_assert(ty == PacketType::Version))]
  Version { version: u32 },
  #[br(pre_assert(ty == PacketType::Auth))]
  Auth {
    #[br(count(size/4))]
    auth_types: Vec<AuthType>,
  },
  #[br(pre_assert(ty == PacketType::GetDriverName))]
  GetDriverName { driver: NullString },
  #[br(pre_assert(ty == PacketType::GetModelId))]
  GetModelId { model: NullString },
  #[br(pre_assert(ty == PacketType::GetDisplaySize))]
  GetDisplaySize { width: u32, height: u32 },
  #[br(pre_assert(ty == PacketType::ParameterValue))]
  ParameterValue {
    #[br(map(|bits: u32| ParameterValueFlags::from_bits_truncate(bits)))]
    #[bw(map(ParameterValueFlags::bits))]
    flags: ParameterValueFlags,
    parameter: u32,
    sub_parameter: u64,
    #[br(count(size-16))]
    value: Vec<u8>,
  },
  #[br(pre_assert(ty == PacketType::ParameterUpdate))]
  ParameterUpdate {
    #[br(map(|bits: u32| ParameterValueFlags::from_bits_truncate(bits)))]
    #[bw(map(ParameterValueFlags::bits))]
    flags: ParameterValueFlags,
    parameter: u32,
    sub_parameter: u64,
    #[br(count(size-16))]
    value: Vec<u8>,
  },
}
impl ServerPacketData {
  fn size(&self) -> usize {
    match self {
      ServerPacketData::Ack => 0,
      ServerPacketData::Error {
        code: _,
        packet_type: _,
        packet_data,
      } => 8 + packet_data.len(),
      ServerPacketData::Exception { packet } => packet.len(),
      ServerPacketData::Key { key: _ } => 8,
      ServerPacketData::Version { version: _ } => 4,
      ServerPacketData::Auth { auth_types } => auth_types.len() / 4,
      ServerPacketData::GetDriverName { driver } => driver.len(),
      ServerPacketData::GetModelId { model } => model.len(),
      ServerPacketData::GetDisplaySize {
        width: _,
        height: _,
      } => 8,
      ServerPacketData::Packet { packet } => packet.len(),
      ServerPacketData::ParameterValue {
        flags: _,
        parameter: _,
        sub_parameter: _,
        value,
      } => 16 + value.len(),
      ServerPacketData::ParameterUpdate {
        flags: _,
        parameter: _,
        sub_parameter: _,
        value,
      } => 16 + value.len(),
    }
  }
}
#[binrw]
#[brw(big)]
#[br(assert(size as usize == data.size()))]
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ClientPacket {
  #[br(temp)]
  #[bw(calc(data.size() as u32))]
  size: u32,
  pub ty: PacketType,
  #[br(args(size, ty))]
  pub data: ClientPacketData,
}
#[binrw]
#[brw(big)]
#[br(assert(size as usize == data.size()))]
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ServerPacket {
  #[br(temp)]
  #[bw(calc(data.size() as u32))]
  size: u32,
  pub ty: PacketType,
  #[br(args(size, ty))]
  pub data: ServerPacketData,
}
