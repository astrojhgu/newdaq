use packed_struct::{prelude::*, types::bits::ByteArray};

use newdaq::ctrl_msg::CommandFrame;

use clap::Parser;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// config
    #[clap(short = 'a', long = "add", value_parser)]
    addr_with_port: String,
}

use std::net::UdpSocket;

fn main() {
    let args = Args::parse();

    let addr = args.addr_with_port;
    let udp = UdpSocket::bind(addr).unwrap();
    loop {
        let mut buffer = vec![0_u8; 1024];
        let s = udp.recv(&mut buffer).unwrap();
        println!("{} bytes received", s);
        let size = <CommandFrame as PackedStruct>::ByteArray::len();
        let cmd = CommandFrame::unpack_from_slice(&buffer[..size]).unwrap();
        let cmd = cmd.get_cmd();
        println!("{}", cmd.cmd_string());
    }
}
