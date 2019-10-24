use block_peers::grid::Grid;
use block_peers::net::{ClientMessage, ServerMessage, Socket};

const GRID_HEIGHT: u32 = 20;
const GRID_WIDTH: u32 = 10;

fn main() {
    let mut socket = Socket::bind("0.0.0.0:4485").expect("could not create socket");
    let grid = Grid::new(GRID_HEIGHT, GRID_WIDTH);

    match socket.receive::<ClientMessage>() {
        Ok((source_addr, ClientMessage::Connect)) => {
            println!("client at {:?} connected", source_addr);
            socket
                .send(source_addr, &ServerMessage::Ack { grid })
                .unwrap();
        }
        Err(_) => {
            println!("received unknown message");
        }
    }
}
