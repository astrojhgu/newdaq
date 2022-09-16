use clap::{Arg, Command};

use std::{
    fs::File
    , io::Write
};

const PKT_LEN:usize=8224;
type data_type=f32;
const NCH:usize=8192;
const NPORT:usize=4;
const NCORR:usize=NPORT*(NPORT+1)/2;
const PORT1:u32=1;
const PORT2:u32=3;
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

    // get the default Device
    let device = pcap::Device::list().unwrap().iter().filter(|&d|{
        d.name==dev_name
    }).nth(0).unwrap().clone();
    println!("Using device {}", device.name);

    // Setup Capture
    let mut cap = pcap::Capture::from_device(device)
        .unwrap()
        .immediate_mode(true)
        .promisc(true)
        .timeout(1000000)

        ;
    

    let mut cap=cap.open().unwrap();
    cap.direction(pcap::Direction::In).unwrap();

    //cap.filter("udp", true).unwrap();
    // get a packet and print its bytes

    let mut old_gcnt=0;
    let mut dropped=false;
    let mut buf=vec![0_u8; 2*NCH*std::mem::size_of::<f32>()];
    loop{
        match cap.next(){
            Ok(pkt) if pkt.data.len()==PKT_LEN=>{
            
            let data=pkt.data;
            let bid1=data[16] as u32;
            let pid1=data[17] as u32;
            let bid2=data[18] as u32;
            let pid2=data[19] as u32;
            let pcnt=data[20] as u32;
            let gcnt=unsafe{*((data.as_ptr().offset(24) as *const u32))};
            let fcnt=unsafe{*((data.as_ptr().offset(28) as *const u32))};
            //eprintln!("{} {} {} {} {} {} {}", bid1, pid1, bid2, pid2, pcnt, gcnt, fcnt);
            //eprintln!("{}", fcnt);
            assert!(bid1==1 && bid2==1 && pcnt<8);
            if old_gcnt+1!=gcnt{
                println!("x");
                dropped=true;
            }

            old_gcnt=gcnt;
            //eprintln!("{} {} {} {} {} {}", bid1, pid1, bid2, pid2, pcnt, gcnt);
            //std::process::exit(0);
            if pcnt==0{
                dropped=false;
            }
            
            if pid1==PORT1 && pid2==PORT2{
                let offset=(pcnt*8192) as usize;
                buf[offset..offset+8192].copy_from_slice(&data[32..]);

                
                println!("{}", pcnt);
                if pcnt==7 && !dropped{
                    let mut dump_file=File::create("a.bin").unwrap();
                    dump_file.write(&buf).unwrap();
                    std::process::exit(0);
                }
            }
        }
        Err(e)=> println!("{:?}", e),
        _=>()
        }
        
        
    }
}

