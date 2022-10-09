use chrono::offset::Local;
use clap::Parser;
use newdaq::{
    ctrl_msg::{CmdEnum, CommandFrame},
    TimeStamp,
};
use packed_struct::{prelude::*, types::bits::ByteArray};

use std::{fs::File, io::Write, net::UdpSocket};

use serde_json::to_writer as to_json;
use serde_yaml::to_writer as to_yaml;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// config
    #[clap(short = 'a', long = "add", value_parser)]
    addr_with_port: String,

    /// If debug
    #[clap(short('d'), long("dbg"), action)]
    dbg_of: Option<String>,
}

fn main() {
    let args = Args::parse();

    let debuging = args.dbg_of.is_some();

    let addr = args.addr_with_port;
    let udp = UdpSocket::bind(addr).unwrap();
    loop {
        let mut buffer = vec![0_u8; 1024];
        let (s, remote_addr) = udp.recv_from(&mut buffer).unwrap();

        let now = Local::now();
        println!("{} bytes received from {} ", s, remote_addr);
        let size = <CommandFrame as PackedStruct>::ByteArray::len();
        let cmd = CommandFrame::unpack_from_slice(&buffer[..size]).unwrap();

        if debuging {
            println!("{:?}", cmd);
        }

        let cmd = cmd.get_cmd();

        let mut outfile = std::fs::File::create("dev_reply.log").unwrap();
        println!("{}", cmd.cmd_string());
        println!("{:?}", now);
        writeln!(&mut outfile, "{}", cmd.cmd_string()).unwrap();
        writeln!(&mut outfile, "{:?}", now).unwrap();

        let timestamp = TimeStamp {
            time: format!("{:?}", now),
        };

        let mut outfile = std::fs::File::create("/dev/shm/last_msg_time.json").unwrap();
        to_json(&mut outfile, &timestamp).unwrap();

        let enum_cmd = cmd.to_enum();
        //create_dir_all("./reply_data").unwrap();

        if let Some(ref dbg_of) = args.dbg_of {
            let mut dbg_file = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(dbg_of)
                .expect("File open failed");
            to_yaml(&mut dbg_file, &enum_cmd).unwrap();
        }

        match enum_cmd {
            CmdEnum::HealthInfo(x) => {
                let mut outfile = File::create("/dev/shm/temperature.yaml").unwrap();
                to_yaml(&mut outfile, &x).unwrap();

                let mut outfile = File::create("/dev/shm/temperature.json").unwrap();
                to_json(&mut outfile, &x).unwrap();
            }
            CmdEnum::WorkMode(x) => {
                let mut outfile = File::create("/dev/shm/mode.yaml").unwrap();
                to_yaml(&mut outfile, &x).unwrap();

                let mut outfile = File::create("/dev/shm/mode.json").unwrap();
                to_json(&mut outfile, &x).unwrap();
            }
            CmdEnum::SelfCheckState(x) => {
                let mut outfile = File::create("/dev/shm/check.yaml").unwrap();
                to_yaml(&mut outfile, &x).unwrap();

                let mut outfile = File::create("/dev/shm/check.json").unwrap();
                to_json(&mut outfile, &x).unwrap();
            }
            CmdEnum::DataState(x) => {
                let mut outfile = File::create("/dev/shm/state.yaml").unwrap();
                to_yaml(&mut outfile, &x).unwrap();

                let mut outfile = File::create("/dev/shm/state.json").unwrap();
                to_json(&mut outfile, &x).unwrap();
            }
            _ => {}
        }
    }
}
