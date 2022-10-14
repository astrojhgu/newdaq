use clap::Parser;

use std::{fs::File, io::Write};

use crossbeam::channel::bounded;

use newdaq::{LaceDataFrame};

const PKT_LEN:usize=8200;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// config
    #[clap(short = 'd', long = "dev", value_parser)]
    dev: String,

    #[clap(short = 'o', long = "out", value_parser)]
    out: String,

    /// If dry run
    #[clap(short('c'), long("cnt"), action)]
    cnt: usize,
}

fn main() {
    let args = Args::parse();

    let dev_name = args.dev;

    let out = args.out;

    let cnt = args.cnt;

    // get the default Device
    let device = pcap::Device::list()
        .unwrap()
        .iter()
        .find(|&d| d.name == dev_name)
        .unwrap()
        .clone();
    println!("Using device {}", device.name);

    // Setup Capture
    let cap = pcap::Capture::from_device(device)
        .unwrap()
        .immediate_mode(false)
        .buffer_size(1024 * 1024 * 1024)
        .promisc(true)
        .timeout(0);

    let mut cap = cap.open().unwrap();
    cap.direction(pcap::Direction::In).unwrap();

    //cap.filter("udp", true).unwrap();
    // get a packet and print its bytes

    let (sender, receiver) = bounded(16384);

    //let exit=Arc::new(Mutex::new(false))

    let _ = std::thread::spawn(move || {
        let mut last_cnt = 0;
        let mut outfile = File::create(out).unwrap();
        let mut i=0;
        let t0=chrono::offset::Local::now();
        let mut pkt_cnt=0;
        loop {
            let frame_buf1: LaceDataFrame = receiver.recv().unwrap();
            
            pkt_cnt+=1;
            if pkt_cnt%100000==0{
                let t1=chrono::offset::Local::now();
                let dur=t1-t0;
                let ms=dur.num_milliseconds();
                let packets_per_sec=pkt_cnt as f64/ms as f64*1000.0;
                let bytes_per_sec=packets_per_sec*8242.00/1e9;
                println!("{} {} {}", pkt_cnt, packets_per_sec, bytes_per_sec);
            }
            if last_cnt + 1 != frame_buf1.cnt && i != 0 {
                print!("dropped {} ", frame_buf1.cnt - last_cnt + 1);
            }
            //eprintln!("{} {} {} {} {} {}", bid1, pid1, bid2, pid2, pcnt, gcnt);
            //std::process::exit(0);
            let _ptr = unsafe {
                std::slice::from_raw_parts(&frame_buf1.payload as *const i16 as *const u8, 8192)
            };
            //outfile.write_all(ptr).unwrap();
            last_cnt = frame_buf1.cnt;
            i+=1;
            if cnt!=0 && i==cnt{
                println!("aa");
                break;
            }
        }
        outfile.flush().unwrap();
        std::process::exit(0);
    });

    let _ = std::thread::spawn(|| loop {
        std::thread::sleep(std::time::Duration::from_secs(1));
        eprintln!(".");
    });

    loop {
        match cap.next_packet() {
            Ok(pkt) if pkt.data.len() == PKT_LEN+42 => {
                let frame_buf1 = LaceDataFrame::from_raw(&pkt.data[42..]);//skip udp head
                sender.send(frame_buf1).unwrap();



            }
            Err(e) => println!("{:?}", e),
            Ok(pkt) => {
                println!("{}", pkt.data.len());
            },
        }
    }
}
