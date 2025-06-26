use binrw::{BinRead, BinWrite};
use brlapi_types::{AuthType, ClientPacket, ClientPacketData, ErrorCode, PacketType, ServerPacket, ServerPacketData};
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
async fn handle_connection(mut socket: TcpStream, auth_key: Option<String>) {
  write_packet(ServerPacket { ty: PacketType::Version, data: ServerPacketData::Version { version: 8 }}, &mut socket).await;
  let version_packet = read_packet(&mut socket).await;
  match version_packet {
    ClientPacket { ty: PacketType::Version, data: ClientPacketData::Version { version: 8 }} => {},
    _ => {
      write_packet(ServerPacket { ty: PacketType::Error, data: ServerPacketData::Error { code: ErrorCode::BadProtocolVersion }}, &mut socket).await;
      return;
    }
  }
  let auth_type = if auth_key.is_some() {
    AuthType::Key
  }
  else {
    AuthType::None
  };
  write_packet(ServerPacket { ty: PacketType::Auth, data: ServerPacketData::Auth { auth_types: vec![auth_type] }}, &mut socket).await;
  if let Some(auth_key) = auth_key {
    loop {
      let auth_packet = read_packet(&mut socket).await;
      match auth_packet {
        ClientPacket { ty: PacketType::Auth, data: ClientPacketData::Auth { auth_type: AuthType::Key, key: client_key }} => {
          if String::from_utf8_lossy(&client_key) == auth_key {
            write_packet(ServerPacket { ty: PacketType::Ack, data: ServerPacketData::Ack }, &mut socket).await;
            break;
          }
          else {
            write_packet(ServerPacket { ty: PacketType::Error, data: ServerPacketData::Error { code: ErrorCode::AuthenticationFailed }}, &mut socket).await;
            continue;
          }
        },
        _ => {
          write_packet(ServerPacket { ty: PacketType::Error, data: ServerPacketData::Error { code: ErrorCode::BadProtocolVersion }}, &mut socket).await;
          return;
        }
      }
    }
  }
}
pub async fn start(port: u16, auth_key: Option<String>) {
  let listener = TcpListener::bind(SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), port)).await.unwrap();
  loop {
    let (socket, _) = listener.accept().await.unwrap();
    let auth_key2 = auth_key.clone();
    tokio::spawn(async move {
      handle_connection(socket, auth_key2).await;
    });
  }
}
