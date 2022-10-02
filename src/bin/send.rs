use packed_struct::prelude::*;

use clap::Parser;

use newdaq::ctrl_msg::{CmdEnum, CommandFrame};
use serde_yaml::from_reader;

use std::{fs::File, net::UdpSocket};

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// config
    #[clap(short = 'a', long = "add", value_parser)]
    addr_with_port: String,

    #[clap(short = 'c', long = "cfg", value_parser)]
    cfg: String,
}

fn main() {
    let args = Args::parse();

    let addr = args.addr_with_port;

    let udp = UdpSocket::bind("192.168.1.50:8888").unwrap();

    let cfg = args.cfg;

    let mut infile = File::open(cfg).expect("File not open");

    let cmds: Vec<CmdEnum> = from_reader(&mut infile).unwrap();

    for cmd in cmds {
        let cmd = cmd.get_cmd();
        let cmd = CommandFrame::from_msg(&*cmd);
        let data = cmd.pack().unwrap();
        udp.send_to(&data, &addr).unwrap();
        std::thread::sleep(std::time::Duration::from_millis(1000));
    }
}
