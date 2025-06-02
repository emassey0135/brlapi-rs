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
}
#[binrw]
#[brw(big, import(size: u32, ty: PacketType))]
#[derive(Debug, PartialEq, Eq, Clone)]
enum PacketData {
  #[br(pre_assert(ty == PacketType::Version))]
  #[brw(assert(size == 4))]
  Version { version: u32 },
  #[br(pre_assert(ty == PacketType::Auth))]
  #[br(assert(size as usize == auth_types.len()*4))]
  AuthRequest {
    #[br(count(size/4))]
    auth_types: Vec<AuthType>,
  },
  #[br(pre_assert(ty == PacketType::Auth))]
  #[br(assert(auth_type == AuthType::Key))]
  #[br(assert(size as usize == key.len()+4))]
  AuthResponse {
    auth_type: AuthType,
    key: NullString,
  },
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
