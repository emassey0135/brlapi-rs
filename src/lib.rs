use binrw::{NullString, binrw};
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
  SetFocus,
  LeaveTtyMode,
}
#[binrw]
#[brw(big, import(size: u32, ty: PacketType))]
#[derive(Debug, PartialEq, Eq, Clone)]
enum PacketData {
  #[br(pre_assert(ty == PacketType::Ack))]
  #[br(pre_assert(size == 0))]
  AckResponse,
  #[br(pre_assert(ty == PacketType::Error))]
  #[brw(assert(size == 4))]
  ErrorResponse { error: u32 },
  #[br(pre_assert(ty == PacketType::Exception))]
  #[brw(assert(size as usize == packet.len()))]
  ExceptionResponse {
    #[br(count(size))]
    packet: Vec<u8>,
  },
  #[br(pre_assert(ty == PacketType::Version))]
  #[brw(assert(size == 4))]
  Version { version: u32 },
  #[br(pre_assert(ty == PacketType::Auth))]
  #[brw(assert(size as usize == auth_types.len()*4))]
  AuthRequest {
    #[br(count(size/4))]
    auth_types: Vec<AuthType>,
  },
  #[br(pre_assert(ty == PacketType::Auth))]
  #[br(assert(auth_type == AuthType::Key))]
  #[bw(assert(*auth_type == AuthType::Key))]
  #[brw(assert(size as usize == key.len()+4))]
  AuthResponse {
    auth_type: AuthType,
    key: NullString,
  },
  #[br(pre_assert(ty == PacketType::GetDriverName))]
  #[br(pre_assert(size == 0))]
  GetDriverNameRequest,
  #[br(pre_assert(ty == PacketType::GetDriverName))]
  #[brw(assert(size as usize == driver.len()))]
  GetDriverNameResponse { driver: NullString },
  #[br(pre_assert(ty == PacketType::GetModelId))]
  #[br(pre_assert(size == 0))]
  GetModelIdRequest,
  #[br(pre_assert(ty == PacketType::GetModelId))]
  #[brw(assert(size as usize == model.len()))]
  GetModelIdResponse { model: NullString },
  #[br(pre_assert(ty == PacketType::GetDisplaySize))]
  #[br(pre_assert(size == 0))]
  GetDisplaySizeRequest,
  #[br(pre_assert(ty == PacketType::GetDisplaySize))]
  #[brw(assert(size == 8))]
  GetDisplaySizeResponse { width: u32, height: u32 },
  #[br(pre_assert(ty == PacketType::EnterTtyMode))]
  #[brw(assert(size == 8+ttys_len+driver_len))]
  #[br(assert(ttys_len as usize == ttys.len()))]
  #[bw(assert(*ttys_len as usize == ttys.len()))]
  #[br(assert(driver_len as usize == driver.len()))]
  #[bw(assert(*driver_len as usize == driver.len()))]
  EnterTtyModeRequest {
    ttys_len: u32,
    #[br(count(ttys_len))]
    ttys: Vec<u32>,
    driver_len: u32,
    #[br(count(driver_len))]
    driver: Vec<u8>,
  },
  #[br(pre_assert(ty == PacketType::SetFocus))]
  #[brw(assert(size == 4))]
  SetFocusRequest { tty: u32 },
  #[br(pre_assert(ty == PacketType::LeaveTtyMode))]
  #[br(pre_assert(size == 0))]
  LeaveTtyModeRequest,
}
#[binrw]
#[brw(big)]
#[derive(Debug, PartialEq, Eq, Clone)]
struct Packet {
  size: u32,
  ty: PacketType,
  #[br(args(size, ty))]
  #[bw(args(*size, *ty))]
  data: PacketData,
}
