use binrw::binrw;
#[binrw]
#[brw(big)]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum PacketType {
  #[brw(magic(b"\0\0\0v"))]
  Version,
}
#[binrw]
#[brw(big, import(size: u32, ty: PacketType))]
#[derive(Debug, PartialEq, Eq, Clone)]
enum PacketData {
  #[br(pre_assert(ty == PacketType::Version), pre_assert(size == 1))]
  Version(u32),
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
