use binrw::{BinRead, BinWrite};
use brlapi_types::{AuthType, ClientPacket, ClientPacketData, ErrorCode, ServerPacket, ServerPacketData};
use brlapi_types::keycode::Keycode;
use louis::Louis;
use ndarray::{Array1, Array2, s};
use std::io::Cursor;
use std::net::{Ipv4Addr, SocketAddrV4};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{mpsc, oneshot};

thread_local! {
  static LOUIS: Louis = Louis::new().unwrap();
}
pub struct ServerBackend {
  pub columns: u8,
  pub lines: u8,
  pub braille_tx: mpsc::Sender<Array2<u8>>,
  pub keycode_rx: mpsc::Receiver<Keycode>
}
struct ServerState {
  columns: u8,
  lines: u8,
  cursor_position: Option<u16>,
  braille_matrix: Array2<u8>
}
enum Command {
  GetDimentions { result_tx: oneshot::Sender<(u8, u8)> },
  SetCursor { position: Option<u16>, result_tx: oneshot::Sender<()> },
  SetBrailleMatrixSection { start: u16, length: u16, braille: Array1<u8>, result_tx: oneshot::Sender<()> }
}
async fn read_packet<T: AsyncRead + Unpin>(reader: &mut T) -> Result<ClientPacket, std::io::Error> {
  let mut buffer: Vec<u8> = vec![0; 4];
  reader.read_exact(&mut buffer).await?;
  let size: usize = u32::from_be_bytes(buffer[0..4].try_into().unwrap()) as _;
  buffer.resize(size + 8, 0);
  reader.read_exact(&mut buffer[4..]).await?;
  println!("Packet read: {:?}", &buffer);
  let mut cursor = Cursor::new(buffer);
  let packet = ClientPacket::read(&mut cursor).unwrap();
  println!("{:?}", packet);
  Ok(packet)
}
async fn write_packet<T: AsyncWrite + Unpin>(packet: ServerPacket, writer: &mut T) -> Result<(), std::io::Error> {
  let mut cursor = Cursor::new(Vec::new());
  packet.write(&mut cursor).unwrap();
  let data = cursor.into_inner();
  println!("Writing packet: {:?}", packet);
  println!("{:?}", data);
  writer.write(&data).await?;
  writer.flush().await?;
  Ok(())
}
async fn handle_state(columns: u8, lines: u8, braille_tx: mpsc::Sender<Array2<u8>>, mut command_rx: mpsc::Receiver<Command>) {
  let mut state = ServerState { columns, lines, cursor_position: None, braille_matrix: Array2::zeros((lines as usize, columns as usize)) };
  let new_matrix = state.braille_matrix.clone();
  braille_tx.send(new_matrix).await.unwrap();
  while let Some(command) = command_rx.recv().await {
    match command {
      Command::GetDimentions { result_tx } => result_tx.send((state.columns, state.lines)).unwrap(),
      Command::SetCursor { position, result_tx } => {
        state.cursor_position = position;
        let mut new_matrix = state.braille_matrix.clone();
        if let Some(position) = position {
          let mut braille_cells = new_matrix.view_mut().into_shape_with_order(lines as usize*columns as usize).unwrap();
          let cell = braille_cells.get_mut(position as usize).unwrap();
          *cell |= 192;
        };
        braille_tx.send(new_matrix).await.unwrap();
        result_tx.send(()).unwrap();
      },
      Command::SetBrailleMatrixSection { start, length, braille, result_tx } => {
        let mut braille_cells = state.braille_matrix.view_mut().into_shape_with_order(lines as usize*columns as usize).unwrap();
        let mut slice = braille_cells.slice_mut(s![start as i32..(start+length) as i32]);
        slice.assign(&braille);
        let mut new_matrix = state.braille_matrix.clone();
        if let Some(cursor_position) = state.cursor_position {
          let mut braille_cells = new_matrix.view_mut().into_shape_with_order(lines as usize*columns as usize).unwrap();
          let cell = braille_cells.get_mut(cursor_position as usize).unwrap();
          *cell |= 192;
        };
        braille_tx.send(new_matrix).await.unwrap();
        result_tx.send(()).unwrap();
      }
    }
  }
}
async fn handle_connection(mut socket: TcpStream, auth_key: Option<String>, command_tx: mpsc::Sender<Command>) -> Result<(), std::io::Error> {
  write_packet(ServerPacket { data: ServerPacketData::Version { version: 8 }}, &mut socket).await?;
  let version_packet = read_packet(&mut socket).await?;
  match version_packet.data {
    ClientPacketData::Version { version: 8 } => {},
    _ => {
      write_packet(ServerPacket { data: ServerPacketData::Error { code: ErrorCode::BadProtocolVersion }}, &mut socket).await?;
      return Ok(());
    }
  }
  let auth_type = if auth_key.is_some() {
    AuthType::Key
  }
  else {
    AuthType::None
  };
  write_packet(ServerPacket { data: ServerPacketData::Auth { auth_types: vec![auth_type] }}, &mut socket).await?;
  if let Some(auth_key) = auth_key {
    loop {
      let auth_packet = read_packet(&mut socket).await?;
      match auth_packet.data {
        ClientPacketData::Auth { auth_type: AuthType::Key, key: client_key } => {
          if String::from_utf8_lossy(&client_key) == auth_key {
            write_packet(ServerPacket { data: ServerPacketData::Ack }, &mut socket).await?;
            break;
          }
          else {
            write_packet(ServerPacket { data: ServerPacketData::Error { code: ErrorCode::AuthenticationFailed }}, &mut socket).await?;
            continue;
          }
        },
        _ => {
          write_packet(ServerPacket { data: ServerPacketData::Error { code: ErrorCode::BadProtocolVersion }}, &mut socket).await?;
          return Ok(());
        }
      }
    }
  }
  let (result_tx, result_rx) = oneshot::channel();
  command_tx.send(Command::GetDimentions { result_tx }).await.unwrap();
  let (columns, lines) = result_rx.await.unwrap();
  loop {
    let packet = read_packet(&mut socket).await?;
    match packet.data {
      ClientPacketData::Write { display_number, region, text, and, or, cursor, charset } => {
        let text = text.as_ref().map(|text| String::from_utf8_lossy(text));
        let region = match (region, text.as_ref()) {
          (Some((start, length)), _) => (start-1, length),
          (None, Some(text)) => (0, text.len() as u32),
          (None, None) => (0, 0)
        };
        let mut braille_cells: Array1<u8> = Array1::zeros(region.1 as usize);
        if let Some(text) = text {
          let cells = LOUIS.with(|louis| louis.translate_simple("en-us-comp8.ctb", &text, false, louis::modes::DOTS_UNICODE)
            .chars()
            .map(|char| (u32::from(char)-10240).try_into().unwrap())
            .collect::<Vec<u8>>());
          braille_cells.assign(&Array1::from(cells));
        };
        if let Some(and) = and {
          braille_cells &= &Array1::from(and);
        };
        if let Some(or) = or {
          braille_cells |= &Array1::from(or);
        };
        if let Some(cursor) = cursor {
          let (result_tx, result_rx) = oneshot::channel();
          let position = if cursor==0 {
            None
          }
          else {
            Some((cursor-1) as u16)
          };
          command_tx.send(Command::SetCursor { position, result_tx }).await.unwrap();
          result_rx.await.unwrap();
        };
        if region.1 != 0 {
          let (result_tx, result_rx) = oneshot::channel();
          command_tx.send(Command::SetBrailleMatrixSection { start: region.0 as u16, length: region.1 as u16, braille: braille_cells, result_tx }).await.unwrap();
          result_rx.await.unwrap();
        };
      },
      ClientPacketData::GetDisplaySize => write_packet(ServerPacket { data: ServerPacketData::GetDisplaySize { width: columns as u32, height: lines as u32 }}, &mut socket).await?,
      _ => write_packet(ServerPacket { data: ServerPacketData::Ack }, &mut socket).await?
    };
  }
}
pub async fn start(port: u16, auth_key: Option<String>, backend: ServerBackend) {
  let listener = TcpListener::bind(SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), port)).await.unwrap();
  let (command_tx, command_rx) = mpsc::channel(32);
  tokio::spawn(async move {
    handle_state(backend.columns, backend.lines, backend.braille_tx, command_rx).await;
  });
  loop {
    let (socket, _) = listener.accept().await.unwrap();
    let auth_key2 = auth_key.clone();
    let command_tx2 = command_tx.clone();
    tokio::spawn(async move {
      let _ = handle_connection(socket, auth_key2, command_tx2).await;
    });
  }
}
