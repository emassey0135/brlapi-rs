use binrw::{BinRead, BinWrite};
use brlapi_types::{AuthType, ClientPacket, ClientPacketData, ErrorCode, ServerPacket, ServerPacketData};
use brlapi_types::keycode::Keycode;
use iconv_native::decode_lossy;
use louis::Louis;
use ndarray::{Array1, Array2, s};
use std::io::Cursor;
use std::net::{Ipv4Addr, SocketAddrV4};
use std::thread;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{mpsc, oneshot};

pub struct ServerBackend {
  pub driver_name: String,
  pub model_id: String,
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
  GetDriverName { result_tx: oneshot::Sender<String> },
  GetModelId { result_tx: oneshot::Sender<String> },
  GetDimentions { result_tx: oneshot::Sender<(u8, u8)> },
  SetCursor { position: Option<u16>, result_tx: oneshot::Sender<()> },
  SetBrailleMatrixSection { start: u16, length: u16, braille: Array1<u8>, result_tx: oneshot::Sender<()> }
}
struct LouisRequest {
  table: String,
  text: String,
  result_tx: oneshot::Sender<String>
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
fn louis_runner(mut request_rx: mpsc::Receiver<LouisRequest>) {
  let louis = Louis::new().unwrap();
  while let Some(request) = request_rx.blocking_recv() {
    let result = louis.translate_simple(&request.table, &request.text, false, ::louis::modes::DOTS_UNICODE);
    request.result_tx.send(result).unwrap();
  }
}
async fn handle_state(backend: ServerBackend, mut command_rx: mpsc::Receiver<Command>) {
  let mut state = ServerState { columns: backend.columns, lines: backend.lines, cursor_position: None, braille_matrix: Array2::zeros((backend.lines as usize, backend.columns as usize)) };
  let new_matrix = state.braille_matrix.clone();
  backend.braille_tx.send(new_matrix).await.unwrap();
  while let Some(command) = command_rx.recv().await {
    match command {
      Command::GetDriverName { result_tx } => result_tx.send(backend.driver_name.clone()).unwrap(),
      Command::GetModelId { result_tx } => result_tx.send(backend.model_id.clone()).unwrap(),
      Command::GetDimentions { result_tx } => result_tx.send((state.columns, state.lines)).unwrap(),
      Command::SetCursor { position, result_tx } => {
        state.cursor_position = position;
        let mut new_matrix = state.braille_matrix.clone();
        if let Some(position) = position {
          let mut braille_cells = new_matrix.view_mut().into_shape_with_order(state.lines as usize*state.columns as usize).unwrap();
          let cell = braille_cells.get_mut(position as usize).unwrap();
          *cell |= 192;
        };
        backend.braille_tx.send(new_matrix).await.unwrap();
        result_tx.send(()).unwrap();
      },
      Command::SetBrailleMatrixSection { start, length, braille, result_tx } => {
        let mut braille_cells = state.braille_matrix.view_mut().into_shape_with_order(state.lines as usize*state.columns as usize).unwrap();
        let mut slice = braille_cells.slice_mut(s![start as i32..(start+length) as i32]);
        slice.assign(&braille);
        let mut new_matrix = state.braille_matrix.clone();
        if let Some(cursor_position) = state.cursor_position {
          let mut braille_cells = new_matrix.view_mut().into_shape_with_order(state.lines as usize*state.columns as usize).unwrap();
          let cell = braille_cells.get_mut(cursor_position as usize).unwrap();
          *cell |= 192;
        };
        backend.braille_tx.send(new_matrix).await.unwrap();
        result_tx.send(()).unwrap();
      }
    }
  }
}
async fn handle_connection(mut socket: TcpStream, auth_key: Option<String>, louis_tx: mpsc::Sender<LouisRequest>, command_tx: mpsc::Sender<Command>) -> Result<(), std::io::Error> {
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
  command_tx.send(Command::GetDriverName { result_tx }).await.unwrap();
  let driver_name = result_rx.await.unwrap();
  let (result_tx, result_rx) = oneshot::channel();
  command_tx.send(Command::GetModelId { result_tx }).await.unwrap();
  let model_id = result_rx.await.unwrap();
  let (result_tx, result_rx) = oneshot::channel();
  command_tx.send(Command::GetDimentions { result_tx }).await.unwrap();
  let (columns, lines) = result_rx.await.unwrap();
  loop {
    let packet = read_packet(&mut socket).await?;
    match packet.data {
      ClientPacketData::Write { display_number, region, text, and, or, cursor, charset } => {
        let text = match (text, charset) {
          (None, _) => None,
          (Some(text), None) => Some(String::from_utf8_lossy(&text).to_string()),
          (Some(text), Some(charset)) => Some(decode_lossy(&text, &String::from_utf8_lossy(&charset)).unwrap())
        };
        let region = match (region, text.as_ref()) {
          (Some((start, length)), _) => (start-1, length),
          (None, Some(text)) => (0, text.len() as u32),
          (None, None) => (0, 0)
        };
        let mut braille_cells: Array1<u8> = Array1::zeros(region.1 as usize);
        if let Some(text) = text {
          let (result_tx, result_rx) = oneshot::channel();
          louis_tx.send(LouisRequest { table: "en-us-comp8.ctb".to_owned(), text: text.to_string(), result_tx }).await.unwrap();
          let cells = result_rx.await.unwrap()
            .chars()
            .map(|char| (u32::from(char)-10240).try_into().unwrap())
            .collect::<Vec<u8>>();
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
      ClientPacketData::GetDriverName => write_packet(ServerPacket { data: ServerPacketData::GetDriverName { driver: driver_name.clone().into() }}, &mut socket).await?,
      ClientPacketData::GetModelId => write_packet(ServerPacket { data: ServerPacketData::GetModelId { model: model_id.clone().into() }}, &mut socket).await?,
      ClientPacketData::GetDisplaySize => write_packet(ServerPacket { data: ServerPacketData::GetDisplaySize { width: columns as u32, height: lines as u32 }}, &mut socket).await?,
      _ => write_packet(ServerPacket { data: ServerPacketData::Ack }, &mut socket).await?
    };
  }
}
pub async fn start(port: u16, auth_key: Option<String>, backend: ServerBackend) {
  let listener = TcpListener::bind(SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), port)).await.unwrap();
  let (command_tx, command_rx) = mpsc::channel(32);
  tokio::spawn(async move {
    handle_state(backend, command_rx).await;
  });
  let (louis_tx, louis_rx) = mpsc::channel(32);
  thread::spawn(move || {
    louis_runner(louis_rx)
  });
  loop {
    let (socket, _) = listener.accept().await.unwrap();
    let auth_key2 = auth_key.clone();
    let louis_tx2 = louis_tx.clone();
    let command_tx2 = command_tx.clone();
    tokio::spawn(async move {
      let _ = handle_connection(socket, auth_key2, louis_tx2, command_tx2).await;
    });
  }
}
