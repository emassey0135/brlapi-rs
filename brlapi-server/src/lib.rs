use binrw::{BinRead, BinWrite};
use brlapi_types::{ClientPacket, ClientPacketData, PacketType, ServerPacket, ServerPacketData};
use std::io::Cursor;
use std::net::{Ipv4Addr, SocketAddrV4};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

async fn read_packet<T: AsyncRead + Unpin>(reader: &mut T) -> ClientPacket {
  let mut buffer: Vec<u8> = vec![0; 4];
  reader.read_exact(&mut buffer).await.unwrap();
  let size: usize = u32::from_be_bytes(buffer[0..4].try_into().unwrap()) as _;
  buffer.resize(size + 8, 0);
  reader.read_exact(&mut buffer[4..]).await.unwrap();
  println!("Packet read: {:?}", &buffer);
  let mut cursor = Cursor::new(buffer);
  ClientPacket::read(&mut cursor).unwrap()
}
async fn write_packet<T: AsyncWrite + Unpin>(packet: ServerPacket, writer: &mut T) {
  let mut cursor = Cursor::new(Vec::new());
  packet.write(&mut cursor).unwrap();
  let data = cursor.into_inner();
  println!("Writing packet: {:?}", packet);
  println!("{:?}", data);
  writer.write(&data).await.unwrap();
  writer.flush().await.unwrap();
}
async fn handle_connection(mut socket: TcpStream) {
  write_packet(ServerPacket { ty: PacketType::Version, data: ServerPacketData::Version { version: 8 }}, &mut socket).await;
}
pub async fn start(port: u16) {
  let listener = TcpListener::bind(SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), port)).await.unwrap();
  loop {
    let (socket, _) = listener.accept().await.unwrap();
    tokio::spawn(async move {
      handle_connection(socket).await;
    });
  }
}
