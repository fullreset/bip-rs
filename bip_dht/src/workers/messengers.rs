use std::net::{SocketAddr, UdpSocket};
use std::sync::mpsc::{self, SyncSender, Receiver};
use std::thread::{self};

const OUTGOING_MESSAGE_CAPACITY: usize = 10000;
const INCOMING_MESSAGE_CAPACITY: usize = 10000;

pub fn create_outgoing_messenger(socket: UdpSocket) -> SyncSender<(Vec<u8>, SocketAddr)> {
	let (send, recv) = mpsc::sync_channel::<(Vec<u8>, SocketAddr)>(OUTGOING_MESSAGE_CAPACITY);
	
	thread::spawn(move || {
		for (message, addr) in recv {
			send_bytes(&socket, &message[..], addr);
		}
		
		info!("bip_dht: Outgoing messenger received a channel hangup, exiting thread...");
	});
	
	send
}

fn send_bytes(socket: &UdpSocket, bytes: &[u8], addr: SocketAddr) {
	let mut bytes_sent = 0;
	
	while bytes_sent != bytes.len() {
		if let Ok(num_sent) = socket.send_to(&bytes[bytes_sent..], addr) {
			bytes_sent += num_sent;
		} else {
			warn!("bip_dht: Outgoing messenger failed to write {} bytes to {}; {} bytes written before error...",
				bytes.len(), addr, bytes_sent);
			break;
		}
	}
}

pub fn create_incoming_messenger(socket: UdpSocket) -> Receiver<(Vec<u8>, SocketAddr)> {
	let (send, recv) = mpsc::sync_channel(INCOMING_MESSAGE_CAPACITY);
	
	thread::spawn(move || {
		let mut channel_is_open = true;
		
		while channel_is_open {
			let mut buffer = vec![0u8; 1500];
			
			if let Ok((size, addr)) = socket.recv_from(&mut buffer) {
				buffer.truncate(size);
				
				channel_is_open = send_message(&send, buffer, addr);
			} else {
				warn!("bip_dht: Incoming messenger failed to receive bytes...");
			}
		}
		
		info!("bip_dht: Incoming messenger received a channel hangup, exiting thread...");
	});
	
	recv
}

fn send_message(send: &SyncSender<(Vec<u8>, SocketAddr)>, bytes: Vec<u8>, addr: SocketAddr) -> bool {
	send.send((bytes, addr)).is_ok()
}