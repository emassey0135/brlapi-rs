use binrw::{NullString, binrw};
#[binrw]
#[brw(big, repr(u32))]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum ErrorCode {
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
enum AuthType {
  #[brw(magic(b"\0\0\0N"))]
  None,
  #[brw(magic(b"\0\0\0K"))]
  Key,
  #[brw(magic(b"\0\0\0C"))]
  Credentials,
}
#[binrw]
#[brw(big)]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum PacketType {
  #[brw(magic(b"\0\0\0A"))]
  Ack,
  #[brw(magic(b"\0\0\0e"))]
  Error,
  #[brw(magic(b"\0\0\0E"))]
  Exception,
  #[brw(magic(b"\0\0\0k"))]
  Key,
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
}
#[binrw]
#[brw(big)]
#[br(import(size: u32, ty: PacketType))]
#[derive(Debug, PartialEq, Eq, Clone)]
enum PacketData {
  #[br(pre_assert(ty == PacketType::Ack))]
  #[br(pre_assert(size == 0))]
  AckResponse,
  #[br(pre_assert(ty == PacketType::Error))]
  #[br(assert(size as usize == 8+packet_data.len()))]
  ErrorResponse {
    code: ErrorCode,
    packet_type: PacketType,
    #[br(count(size-8))]
    packet_data: Vec<u8>,
  },
  #[br(pre_assert(ty == PacketType::Exception))]
  #[br(assert(size as usize == packet.len()))]
  ExceptionResponse {
    #[br(count(size))]
    packet: Vec<u8>,
  },
  #[br(pre_assert(ty == PacketType::Key))]
  #[br(assert(size == 8))]
  Key { key: u64 },
  #[br(pre_assert(ty == PacketType::Version))]
  #[br(assert(size == 4))]
  Version { version: u32 },
  #[br(pre_assert(ty == PacketType::Auth))]
  #[br(assert(size as usize == auth_types.len()*4))]
  AuthRequest {
    #[br(count(size/4))]
    auth_types: Vec<AuthType>,
  },
  #[br(pre_assert(ty == PacketType::Auth))]
  #[br(assert(auth_type == AuthType::Key))]
  #[bw(assert(*auth_type == AuthType::Key))]
  #[br(assert(size as usize == key.len()+4))]
  AuthResponse {
    auth_type: AuthType,
    key: NullString,
  },
  #[br(pre_assert(ty == PacketType::GetDriverName))]
  #[br(pre_assert(size == 0))]
  GetDriverNameRequest,
  #[br(pre_assert(ty == PacketType::GetDriverName))]
  #[br(assert(size as usize == driver.len()))]
  GetDriverNameResponse { driver: NullString },
  #[br(pre_assert(ty == PacketType::GetModelId))]
  #[br(pre_assert(size == 0))]
  GetModelIdRequest,
  #[br(pre_assert(ty == PacketType::GetModelId))]
  #[br(assert(size as usize == model.len()))]
  GetModelIdResponse { model: NullString },
  #[br(pre_assert(ty == PacketType::GetDisplaySize))]
  #[br(pre_assert(size == 0))]
  GetDisplaySizeRequest,
  #[br(pre_assert(ty == PacketType::GetDisplaySize))]
  #[br(assert(size == 8))]
  GetDisplaySizeResponse { width: u32, height: u32 },
  #[br(pre_assert(ty == PacketType::EnterTtyMode))]
  #[br(assert(size as usize == 8+ttys.len()+driver.len()))]
  #[br(assert(ttys_len as usize == ttys.len()))]
  #[br(assert(driver_len as usize == driver.len()))]
  EnterTtyModeRequest {
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
  #[br(assert(size == 4))]
  SetFocusRequest { tty: u32 },
  #[br(pre_assert(ty == PacketType::LeaveTtyMode))]
  #[br(pre_assert(size == 0))]
  LeaveTtyModeRequest,
  #[br(pre_assert(ty == PacketType::IgnoreKeyRanges))]
  #[br(assert(size as usize == ranges.len()*16))]
  IgnoreKeyRangesRequest {
    #[br(count(size/16))]
    ranges: Vec<(u64, u64)>,
  },
  #[br(pre_assert(ty == PacketType::AcceptKeyRanges))]
  #[br(assert(size as usize == ranges.len()*16))]
  AcceptKeyRangesRequest {
    #[br(count(size/16))]
    ranges: Vec<(u64, u64)>,
  },
}
impl PacketData {
  fn size(&self) -> usize {
    match self {
      PacketData::AckResponse => 0,
      PacketData::ErrorResponse {
        code: _,
        packet_type: _,
        packet_data,
      } => 8 + packet_data.len(),
      PacketData::ExceptionResponse { packet } => packet.len(),
      PacketData::Key { key: _ } => 8,
      PacketData::Version { version: _ } => 4,
      PacketData::AuthRequest { auth_types } => auth_types.len() / 4,
      PacketData::AuthResponse { auth_type: _, key } => key.len() + 4,
      PacketData::GetDriverNameRequest => 0,
      PacketData::GetDriverNameResponse { driver } => driver.len(),
      PacketData::GetModelIdRequest => 0,
      PacketData::GetModelIdResponse { model } => model.len(),
      PacketData::GetDisplaySizeRequest => 0,
      PacketData::GetDisplaySizeResponse {
        width: _,
        height: _,
      } => 8,
      PacketData::EnterTtyModeRequest { ttys, driver } => 8 + ttys.len() + driver.len(),
      PacketData::SetFocusRequest { tty: _ } => 4,
      PacketData::LeaveTtyModeRequest => 0,
      PacketData::IgnoreKeyRangesRequest { ranges } => ranges.len() * 16,
      PacketData::AcceptKeyRangesRequest { ranges } => ranges.len() * 16,
    }
  }
}
#[binrw]
#[brw(big)]
#[br(assert(size as usize == data.size()))]
#[derive(Debug, PartialEq, Eq, Clone)]
struct Packet {
  #[br(temp)]
  #[bw(calc(data.size() as u32))]
  size: u32,
  ty: PacketType,
  #[br(args(size, ty))]
  data: PacketData,
}
