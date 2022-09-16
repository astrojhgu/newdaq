use std::slice::SliceIndex;

use clap::{Arg, Command};

use etherparse::{
    Ethernet2HeaderSlice, InternetSlice, LinkSlice, SlicedPacket, TcpOptionElement,
    TcpOptionReadError, TransportSlice,
};
use pnet::datalink::{channel, interfaces, Channel, ChannelType, Config};

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

    let (mut tx, mut rx) =
        if let Channel::Ethernet(tx, rx) = channel(&dev, cfg).expect("canot open channel") {
            (tx, rx)
        } else {
            panic!();
        };

    let x = rx.next().unwrap();
    let sliced_packet = SlicedPacket::from_ethernet(x);

    match sliced_packet {
        Err(value) => println!("Err {:?}", value),
        Ok(value) => {
            println!("Ok");
            use etherparse::InternetSlice::*;
            use etherparse::LinkSlice::*;
            use etherparse::TransportSlice::*;
            use etherparse::VlanSlice::*;

            match value.link {
                Some(Ethernet2(value)) => println!(
                    "  Ethernet2 {:?} => {:?}",
                    value.source(),
                    value.destination()
                ),
                None => {}
            }

            match value.vlan {
                Some(SingleVlan(value)) => println!("  SingleVlan {:?}", value.vlan_identifier()),
                Some(DoubleVlan(value)) => println!(
                    "  DoubleVlan {:?}, {:?}",
                    value.outer().vlan_identifier(),
                    value.inner().vlan_identifier()
                ),
                None => {}
            }

            match value.ip {
                Some(Ipv4(value, extensions)) => {
                    println!(
                        "  Ipv4 {:?} => {:?}",
                        value.source_addr(),
                        value.destination_addr()
                    );
                    if false == extensions.is_empty() {
                        println!("    {:?}", extensions);
                    }
                }
                Some(Ipv6(value, extensions)) => {
                    println!(
                        "  Ipv6 {:?} => {:?}",
                        value.source_addr(),
                        value.destination_addr()
                    );
                    if false == extensions.is_empty() {
                        println!("    {:?}", extensions);
                    }
                }
                None => {}
            }
            println!("payload len: {}", value.payload.len());
            match value.transport {
                Some(Udp(value)) => {
                    println!(
                        "  UDP {:?} -> {:?}",
                        value.source_port(),
                        value.destination_port()
                    );
                    println!("{:?}", value.to_header());
                }
                Some(Tcp(value)) => {
                    println!(
                        "  TCP {:?} -> {:?}",
                        value.source_port(),
                        value.destination_port()
                    );
                    let options: Vec<Result<TcpOptionElement, TcpOptionReadError>> =
                        value.options_iterator().collect();
                    println!("    {:?}", options);
                }
                Some(Unknown(ip_protocol)) => {
                    println!("  Unknwon Protocol (ip protocol number {:?}", ip_protocol)
                }
                None => {}
            }
        }
    }
}
