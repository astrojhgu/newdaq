use clap::Parser;
use newdaq::ctrl_msg::CommandFrame;
use packed_struct::{prelude::*, types::bits::ByteArray};
use std::net::UdpSocket;
use std::io::Write;
use chrono::offset::Local;
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// config
    #[clap(short = 'a', long = "add", value_parser)]
    addr_with_port: String,
}

fn main() {
    let args = Args::parse();

    let addr = args.addr_with_port;
    let udp = UdpSocket::bind(addr).unwrap();
    loop {
        let mut buffer = vec![0_u8; 1024];
        let (s, remote_addr) = udp.recv_from(&mut buffer).unwrap();

        let now=Local::now();
        println!("{} bytes received from {} ", s, remote_addr);
        let size = <CommandFrame as PackedStruct>::ByteArray::len();
        let cmd = CommandFrame::unpack_from_slice(&buffer[..size]).unwrap();
        let cmd = cmd.get_cmd();

        let mut outfile=std::fs::File::create("dev_reply.log").unwrap();
        println!("{}", cmd.cmd_string());
        println!("{:?}", now);
        writeln!(&mut outfile, "{}", cmd.cmd_string()).unwrap();
        writeln!(&mut outfile, "{:?}", now).unwrap();
    }
}
