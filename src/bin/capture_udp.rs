use std::slice::SliceIndex;

use clap::{Arg, Command};

use etherparse::{
    Ethernet2HeaderSlice, InternetSlice, LinkSlice, SlicedPacket, TcpOptionElement,
    TcpOptionReadError, TransportSlice,
};
use pnet::{
    datalink::{channel, interfaces, Channel, ChannelType, Config},
    packet::{
        ip::IpNextHeaderProtocols::{Udp, UdpLite},
        Packet,
    },
    transport::{transport_channel, udp_packet_iter, TransportChannelType, TransportProtocol},
};

use newdaq::{mac2array, str2macarray};
const SPEC_PER_SEC: usize = 122071;

fn main() {
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

    let ch = transport_channel(
        65536,
        TransportChannelType::Layer4(TransportProtocol::Ipv4(Udp)),
    );
    let (_tx, mut rx) = match ch {
        Ok((tx, rx)) => (tx, rx),
        Err(x) => {
            eprintln!("{:?}", x);
            panic!()
        }
    };

    let mut udp_iter = udp_packet_iter(&mut rx);
    while let Ok((pkt, ip)) = udp_iter.next() {
        println!("{:?} {:?} {}", pkt, ip, pkt.payload().len());
    }
}
