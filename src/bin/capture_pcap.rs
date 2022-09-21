use serde::{Serialize, Deserialize};

use serde_yaml::from_reader;

use clap::{Arg, Command, Parser};

use num_complex::Complex;

use std::{
    fs::File, io::Write
};

use chrono::offset::Local;

use crossbeam::channel::bounded;

use newdaq::{DataFrame, MetaData, NCH, NCH_PER_PKT, NCORR, PKT_LEN, NPORT_PER_BD};


/// Simple program to greet a person
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
   /// Name of the person to greet
   #[clap(short='c', long="cfg", value_parser)]
   cfg: String
}

#[derive(Clone, Deserialize, Serialize, Debug)]
struct Cfg{
    dev: String
    , out_dir: Vec<std::path::PathBuf>
    , gbytes_per_day: usize
    , stations: Vec<String>
}


fn main() {
    let args=Args::parse();

    let mut cfg_file=std::fs::File::open(args.cfg).unwrap();

    let cfg:Cfg=from_reader(&mut cfg_file).unwrap();

    assert_eq!(cfg.stations.len(), 40);

    println!("{:?}", cfg);

    let dev_name = cfg.dev;

    // get the default Device
    let device = pcap::Device::list()
        .unwrap()
        .iter()
        .filter(|&d| d.name == dev_name)
        .nth(0)
        .unwrap()
        .clone();
    println!("Using device {}", device.name);

    // Setup Capture
    let cap = pcap::Capture::from_device(device)
        .unwrap()
        .immediate_mode(false)
        .buffer_size(1024*1024*1024)
        .promisc(true)
        .timeout(0);

    let mut cap = cap.open().unwrap();
    cap.direction(pcap::Direction::In).unwrap();

    //cap.filter("udp", true).unwrap();
    // get a packet and print its bytes

    

    let mut dropped = false;

    let mut data_buf = vec![Complex::<f32>::default(); NCH * NCORR];

    let (sender, receiver)=bounded(1024);

    let _=std::thread::spawn(move ||{
        let mut last_meta_data = MetaData::default();
        let mut corr_prod=vec![(0,0); NCORR];
        let mut now=Local::now();
        loop{
            let mut frame_buf1:DataFrame=receiver.recv().unwrap();

            if last_meta_data.gcnt + 1 != frame_buf1.meta_data.gcnt {
                dropped = true;
            }

            //eprintln!("{} {} {} {} {} {}", bid1, pid1, bid2, pid2, pcnt, gcnt);
            //std::process::exit(0);
            if frame_buf1.meta_data.fcnt == 0 && frame_buf1.meta_data.pcnt == 0 {
                if !dropped {
                    //write data
                    let mut outfile=File::create("./a.bin").unwrap();
                    let disk_data=unsafe{std::slice::from_raw_parts(data_buf.as_ptr() as *const u8, data_buf.len()*std::mem::size_of::<Complex<f32>>())};
                    outfile.write(disk_data).unwrap();
                    data_buf.iter_mut().for_each(|x| *x=Complex::default());

                    let mut corr_prod_file=File::create("corr_prod.txt").unwrap();
                    for (i, p) in corr_prod.iter().enumerate(){
                        writeln!(&mut corr_prod_file, "{} {} {}", i, p.0, p.1).unwrap();
                    }
                }
                else{
                    println!("Data dropped, skip writting");
                }
                assert_eq!(frame_buf1.meta_data.gcnt % NCORR as u32, 0);
                now=Local::now();
                println!("new data arrived {} @ {:?}", frame_buf1.meta_data.gcnt, now);
                dropped = false;
            }

            let offset = frame_buf1.meta_data.fcnt as usize * NCH
                + frame_buf1.meta_data.pcnt as usize * NCH_PER_PKT;
            let port_id1=frame_buf1.meta_data.bid1 as usize*NPORT_PER_BD+frame_buf1.meta_data.pid1 as usize;
            let port_id2=frame_buf1.meta_data.bid2 as usize*NPORT_PER_BD+frame_buf1.meta_data.pid2 as usize;

            if frame_buf1.meta_data.bid1!=frame_buf1.meta_data.bid2{
                frame_buf1.payload.chunks_exact_mut(2).for_each(|x| {
                    let y=x[0];
                    x[0]=x[1];
                    x[1]=y;
                })
            }

            
            data_buf[offset..offset+NCH_PER_PKT].clone_from_slice(&frame_buf1.payload);

            if port_id2<port_id1{

                data_buf[offset..offset+NCH_PER_PKT].iter_mut().for_each(|x|{
                    x.im=-x.im;
                });
                corr_prod[frame_buf1.meta_data.fcnt as usize]=(port_id2, port_id1);
            }else{
                corr_prod[frame_buf1.meta_data.fcnt as usize]=(port_id1, port_id2);
            }

            

            last_meta_data = frame_buf1.meta_data;
        }
    });


    loop {
        match cap.next_packet() {
            Ok(pkt) if pkt.data.len() == PKT_LEN => {
                let mut frame_buf1 = DataFrame::default();

                let frame_buf_ptr = unsafe {
                    std::slice::from_raw_parts_mut(
                        (&mut frame_buf1) as *mut DataFrame as *mut u8,
                        std::mem::size_of::<DataFrame>(),
                    )
                };

                let data = pkt.data;

                frame_buf_ptr.clone_from_slice(data);

                sender.send(frame_buf1);

                
                //println!("{} {} {}", pcnt, fcnt, gcnt);
            }
            Err(e) => println!("{:?}", e),
            _ => (),
        }
    }
}
