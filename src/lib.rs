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
}
#[binrw]
#[brw(big, import(size: u32, ty: PacketType))]
#[derive(Debug, PartialEq, Eq, Clone)]
enum PacketData {
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
