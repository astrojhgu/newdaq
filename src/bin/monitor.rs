#![feature(proc_macro_hygiene, decl_macro)]

use std::{
    fs::{create_dir_all, File},
    io::Read,
    mem::size_of,
    path::PathBuf,
    collections::BTreeMap, 
};

use chrono::offset::Local;

use newdaq::{Cfg, NCH, NCORR};

use num_complex::{Complex32};
use plotters::prelude::*;
use serde_yaml::from_reader;

use serde::Deserialize;

const IMG_DIR_STR: &str = "/dev/shm/imgs";
const DATA_FILE_NAME: &str = "/dev/shm/dump.bin";
const CORR_PROD_FILE_NAME: &str = "/dev/shm/corr_prod.txt";
const CFG_FILE_NAME: &str = "/dev/shm/cfg.yaml";

#[derive(Debug, Deserialize, Clone, Copy)]
struct CpRecord {
    pub nb: usize,
    pub a1: usize,
    pub a2: usize,
}

fn read_payload(fname: &str) -> Vec<Complex32> {
    loop {
        if let Ok(mut infile) = File::open(fname) {
            let mut buffer = vec![Complex32::new(0.0, 0.0); NCH * NCORR];
            let data = unsafe {
                std::slice::from_raw_parts_mut(
                    buffer.as_mut_ptr() as *mut u8,
                    buffer.len() * size_of::<Complex32>(),
                )
            };

            if infile.read_exact(data).is_ok() {
                break buffer;
            } else {
                println!("file not ready, try again after 100 ms");
            }
        } else {
            println!("file not opened, try again after 100 ms");
        }
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
}

fn read_corr_prod(fname: &str) -> Vec<CpRecord> {
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(false)
        .delimiter(b' ')
        .from_reader(loop {
            if let Ok(f) = File::open(fname) {
                break f;
            }
        });
    loop {
        let result = rdr
            .deserialize()
            .filter(|x| x.is_ok())
            .map(|x: Result<CpRecord, _>| x.unwrap())
            .collect::<Vec<_>>();
        if result.len() == NCORR {
            break result;
        }
    }
}

fn plot_spec(a1: &str, a2: &str, payload: &[Complex32], corr_prod_map: &BTreeMap<(usize,usize),usize>, station_ant_map: &BTreeMap<String, usize>, freq: &[f32]){
    let fname=a1.to_string()+a2;
    let a1=station_ant_map[a1];
    let a2=station_ant_map[a2];

    let spec_id=if a2>=a1 {corr_prod_map[&(a1,a2)]} else {corr_prod_map[&(a2,a1)]};
    let data = &payload[spec_id * NCH..(spec_id + 1) * NCH];

    let spec: Vec<_> = data.iter().map(|x| x.norm().log10() * 10.0).collect();
    let phase: Vec<_> = data.iter().map(|x| x.arg().to_degrees()).collect();

    //println!("{:?}", spec);

    let img_out_dir = PathBuf::from(IMG_DIR_STR);

    let out_img_name = img_out_dir.join(fname.clone()+"_ampl.png");
    let root = BitMapBackend::new(&out_img_name, (640, 480)).into_drawing_area();
    root.fill(&WHITE).unwrap();
    let mut chart = ChartBuilder::on(&root)
        .caption(&fname, ("sans-serif", 50).into_font())
        .margin(5)
        .x_label_area_size(30)
        .y_label_area_size(30)
        .build_cartesian_2d(freq[0]..freq[freq.len() - 1], 50_f32..100_f32)
        .unwrap();
    chart.configure_mesh().draw().unwrap();

    chart
    .draw_series(LineSeries::new(freq.iter().cloned().zip(spec.iter().cloned()),
        &RED,
    )).unwrap();
    root.present().unwrap();

    let out_img_name = img_out_dir.join(fname.clone()+"_phase.png");
    let root = BitMapBackend::new(&out_img_name, (640, 480)).into_drawing_area();
    root.fill(&WHITE).unwrap();
    let mut chart = ChartBuilder::on(&root)
        .caption(fname, ("sans-serif", 50).into_font())
        .margin(5)
        .x_label_area_size(30)
        .y_label_area_size(30)
        .build_cartesian_2d(freq[0]..freq[freq.len() - 1], -185_f32..185_f32)
        .unwrap();
    chart.configure_mesh().draw().unwrap();

    chart
    .draw_series(LineSeries::new(freq.iter().cloned().zip(phase.iter().cloned()),
        &BLUE,
    )).unwrap();
    root.present().unwrap();
}

fn update() {
    create_dir_all(IMG_DIR_STR).unwrap();
    let freq: Vec<_> = (0..NCH).map(|i| i as f32 * 200_f32 / NCH as f32).collect();

    let cfg: Cfg = from_reader(File::open(CFG_FILE_NAME).unwrap()).unwrap();

    let stations=cfg.stations;
    let station_ant_map=BTreeMap::from_iter(stations.iter().cloned().enumerate().map(|(aid, aname)|{
        (aname, aid)
    }));

    let payload = read_payload(DATA_FILE_NAME);

    let corr_prod = read_corr_prod(CORR_PROD_FILE_NAME);
    let corr_prod_map=BTreeMap::from_iter(corr_prod.iter().cloned().map(|x|{
        ((x.a1,x.a2), x.nb)
    }));
    

    for aid in &stations{
        plot_spec(aid,aid,&payload, &corr_prod_map,&station_ant_map, &freq);
    }

    for aid in &stations[1..]{
        plot_spec("E01",aid,&payload, &corr_prod_map,&station_ant_map, &freq);
    }
}

fn main(){
    loop{
        update();
        let now=Local::now();
        println!("{:?}", now);
        std::thread::sleep(std::time::Duration::from_secs(10));
    }
}