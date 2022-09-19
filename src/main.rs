use std::slice::SliceIndex;

use clap::{Arg, Command};

use etherparse::{
    Ethernet2HeaderSlice, InternetSlice, LinkSlice, SlicedPacket, TcpOptionElement,
    TcpOptionReadError, TransportSlice,
};
use pnet::datalink::{channel, interfaces, Channel, ChannelType, Config};

use newdaq::{mac2array, str2macarray, DataFrame, PKT_LEN, MetaData};
const SPEC_PER_SEC: usize = 122071;

fn main() {
    println!("{}", std::mem::size_of::<newdaq::MetaData>());
    println!("{}", std::mem::size_of::<newdaq::DataFrame>());
    let matches = Command::new("capture")
        .arg(
            Arg::new("dev_name")
                .short('d')
                .long("dev")
                .takes_value(true)
                .value_name("dev name")
                .required(true),
        )
        .get_matches();

    let dev_name = matches.value_of("dev_name").unwrap();
    let dev = interfaces()
        .into_iter()
        .filter(|x| x.name == dev_name)
        .nth(0)
        .expect("Cannot find dev");
    let local_mac = mac2array(&dev.mac.unwrap());
    let cfg = Config {
        write_buffer_size: 65536,
        read_buffer_size: (1 << 32),
        read_timeout: None,
        write_timeout: None,
        channel_type: ChannelType::Layer2,
        bpf_fd_attempts: 1000,
        linux_fanout: None,
        promiscuous: true,
    };

    let (mut _tx, mut rx) =
        if let Channel::Ethernet(tx, rx) = channel(&dev, cfg).expect("canot open channel") {
            (tx, rx)
        } else {
            panic!();
        };

    let mut frame_buf=DataFrame::default();

    let frame_buf_ptr=unsafe{std::slice::from_raw_parts_mut((&mut frame_buf) as *mut DataFrame as *mut u8, std::mem::size_of::<DataFrame>())};
    let mut last_meta_data=MetaData::default();        
    loop {
        let x = rx.next().unwrap();
        if x.len()==PKT_LEN{
            frame_buf_ptr.clone_from_slice(x);
            //eprintln!("{} {} {} {} {} {}", frame_buf.meta_data.bid1, frame_buf.meta_data.pid1, frame_buf.meta_data.bid2, frame_buf.meta_data.pid2, frame_buf.meta_data.fcnt, frame_buf.meta_data.gcnt);

            if last_meta_data.gcnt+1!=frame_buf.meta_data.gcnt{
                println!("{} pkts dropped", frame_buf.meta_data.gcnt-1-last_meta_data.gcnt);
            }

            if frame_buf.meta_data.fcnt==819{
                break;
            }
            last_meta_data=frame_buf.meta_data;
        }        
    }
}
