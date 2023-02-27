#![feature(proc_macro_hygiene, decl_macro)]

use std::{
    collections::BTreeMap,
    fs::{create_dir_all, File},
    io::Read,
    mem::size_of,
    path::PathBuf,
};

use named_lock::NamedLock;

use chrono::offset::Local;

use newdaq::{Cfg, NANTS, NCH, NCORR};

use num_complex::Complex32;
use plotters::prelude::*;
use serde_yaml::from_reader;

use serde::Deserialize;

const IMG_DIR_STR: &str = "/dev/shm/imgs";
const DATA_FILE_NAME: &str = "/dev/shm/dump.bin";
const CORR_PROD_FILE_NAME: &str = "/dev/shm/corr_prod.txt";
const CFG_FILE_NAME: &str = "/dev/shm/cfg.yaml";
const DEBUG: bool = false;

#[derive(Debug, Deserialize, Clone, Copy)]
struct CpRecord {
    pub nb: usize,
    pub a1: usize,
    pub a2: usize,
}

fn read_payload(fname: &str) -> Vec<Complex32> {
    let lock = NamedLock::create("daq_dump_lock").unwrap();
    loop {
        {
            let _guard = lock.lock().unwrap();
            if DEBUG {
                std::fs::copy("/dev/shm/dump.bin", "/dev/shm/back/dump.bin").unwrap();
                std::fs::copy("/dev/shm/corr_prod.txt", "/dev/shm/back/corr_prod.txt").unwrap();
            }
            //
            print!("L:{} ", fname);
            if let Ok(mut infile) = File::open(fname) {
                let mut buffer = vec![Complex32::new(0.0, 0.0); NCH * NCORR];
                let data = unsafe {
                    std::slice::from_raw_parts_mut(
                        buffer.as_mut_ptr() as *mut u8,
                        buffer.len() * size_of::<Complex32>(),
                    )
                };

                if infile.read_exact(data).is_ok() {
                    print!("R");
                    break buffer;
                } else {
                    println!("file not ready, try again after 100 ms");
                }
            } else {
                println!("file not opened, try again after 100 ms");
            }
        }
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
}

fn read_corr_prod(fname: &str) -> Vec<CpRecord> {
    let lock = NamedLock::create("daq_dump_lock").unwrap();
    loop {
        {
            let _guard = lock.lock().unwrap();
            print!("L");
            let mut rdr = csv::ReaderBuilder::new()
                .has_headers(false)
                .delimiter(b' ')
                .from_reader(loop {
                    if let Ok(f) = File::open(fname) {
                        break f;
                    }
                });
            let result = rdr
                .deserialize()
                .filter_map(|x| x.ok())
                //.filter(|x| x.is_ok())
                //.map(|x: Result<CpRecord, _>| x.unwrap())
                .collect::<Vec<_>>();
            if result.len() == NCORR {
                println!("R");
                break result;
            } else {
                println!("corr prod not ready");
                std::thread::sleep(std::time::Duration::from_millis(100));
            }
        }
    }
}

fn plot_spec(
    a1: &str,
    a2: &str,
    payload: &[Complex32],
    corr_prod_map: &BTreeMap<(usize, usize), usize>,
    station_ant_map: &BTreeMap<String, usize>,
    freq: &[f32],
) {
    let fname = a1.to_string() + a2;
    let a1 = station_ant_map[a1];
    let a2 = station_ant_map[a2];

    let spec_id = if a2 >= a1 {
        corr_prod_map[&(a1, a2)]
    } else {
        corr_prod_map[&(a2, a1)]
    };
    let data = &payload[spec_id * NCH..(spec_id + 1) * NCH];

    let spec: Vec<_> = data.iter().map(|x| x.norm().log10() * 10.0).collect();
    let phase: Vec<_> = data.iter().map(|x| x.arg().to_degrees()).collect();

    //println!("{:?}", spec);

    let img_out_dir = PathBuf::from(IMG_DIR_STR);

    let out_img_name = img_out_dir.join(fname.clone() + "_ampl.png");
    let root = BitMapBackend::new(&out_img_name, (800, 600)).into_drawing_area();
    root.fill(&WHITE).unwrap();
    let now = Local::now();
    let caption = format!("{}-{}_ampl", fname, now.format("%m/%d-%T"));
    let mut chart = ChartBuilder::on(&root)
        .caption(&caption, ("sans-serif", 50).into_font())
        .margin(5)
        .x_label_area_size(30)
        .y_label_area_size(30)
        .build_cartesian_2d(freq[0]..freq[freq.len() - 1], 50_f32..120_f32)
        .unwrap();
    chart.configure_mesh().draw().unwrap();

    chart
        .draw_series(LineSeries::new(
            freq.iter().cloned().zip(spec.iter().cloned()),
            RED,
        ))
        .unwrap();
    root.present().unwrap();

    let out_img_name = img_out_dir.join(fname.clone() + "_phase.png");
    let root = BitMapBackend::new(&out_img_name, (800, 600)).into_drawing_area();
    root.fill(&WHITE).unwrap();
    let caption = format!("{}-{}_phase", fname, now.format("%m/%d-%T"));
    let mut chart = ChartBuilder::on(&root)
        .caption(caption, ("sans-serif", 50).into_font())
        .margin(5)
        .x_label_area_size(30)
        .y_label_area_size(30)
        .build_cartesian_2d(freq[0]..freq[freq.len() - 1], -180_f32..180_f32)
        .unwrap();
    chart.configure_mesh().draw().unwrap();

    chart
        .draw_series(LineSeries::new(
            freq.iter().cloned().zip(phase.iter().cloned()),
            BLUE,
        ))
        .unwrap();
    root.present().unwrap();
}

fn plot_spec_all(
    payload: &[Complex32],
    corr_prod_map: &BTreeMap<(usize, usize), usize>,
    _station_ant_map: &BTreeMap<String, usize>,
    freq: &[f32],
) {
    let fname = "spec_all".to_string();

    let spec: Vec<_> = payload.iter().map(|x| x.norm().log10() * 10.0).collect();
    let phase: Vec<_> = payload.iter().map(|x| x.arg().to_degrees()).collect();

    let mut matrix = [[1.0_f32; NANTS]; NANTS];

    let mut vmax = 0_f32;

    for i in 0..NANTS {
        for j in i..NANTS {
            let spec_id = corr_prod_map[&(i, j)];
            let pwr: f32 = payload[spec_id * NCH..(spec_id + 1) * NCH]
                .iter()
                .map(|x| x.norm())
                .skip(NCH / 4)
                .sum();
            matrix[i][j] = pwr;
            vmax = vmax.max(pwr);
        }
    }

    //println!("{:?}", spec);
    let now = Local::now();
    let img_out_dir = PathBuf::from(IMG_DIR_STR);

    let out_img_name = img_out_dir.join(fname + ".png");
    let title = format!("{}", now.format("%m/%d-%T"));
    let root = BitMapBackend::new(&out_img_name, (2000, 2500)).into_drawing_area();
    root.fill(&WHITE).unwrap();

    let root = root
        .titled(&title, ("sans-serif", 100).into_font())
        .unwrap();
    //root_ampl.draw_text(&title, &("sans-serif", 250).into_text_style(&root_ampl), (200,200)).unwrap();

    let drawing_areas_ampl = root.split_evenly((2 * NANTS, NANTS));

    for (n, root1) in drawing_areas_ampl.iter().enumerate() {
        let i = n / NANTS;
        let j = n % NANTS;
        if i / 2 > j {
            continue;
        }

        let ncorr = corr_prod_map[&(i / 2, j)];

        let v = matrix[i / 2][j];

        if i % 2 == 0 {
            let c = HSLColor(0.45, 0.7, (v / vmax) as f64);
            root1.fill(&c).unwrap();
            let data = &spec[ncorr * NCH..(ncorr + 1) * NCH];
            let mut chart = ChartBuilder::on(root1)
                .margin(5)
                .build_cartesian_2d(freq[0]..freq[freq.len() - 1], 50_f32..120_f32)
                .unwrap();
            //chart.configure_mesh().draw().unwrap();

            chart
                .draw_series(LineSeries::new(
                    freq.iter().cloned().zip(data.iter().cloned()),
                    RED,
                ))
                .unwrap();
        } else {
            let c = HSLColor(0.6, 0.7, (v / vmax) as f64);
            root1.fill(&c).unwrap();
            let data = &phase[ncorr * NCH..(ncorr + 1) * NCH];
            let mut chart = ChartBuilder::on(root1)
                .margin(5)
                .build_cartesian_2d(freq[0]..freq[freq.len() - 1], -180_f32..180_f32)
                .unwrap();
            //chart.configure_mesh().draw().unwrap();

            chart
                .draw_series(LineSeries::new(
                    freq.iter()
                        .cloned()
                        .zip(data.iter().cloned())
                        .skip(NCH / 4 + 100)
                        .take(NCH * 3 / 4 - 200),
                    BLUE,
                ))
                .unwrap();
        }

        //root1.present().unwrap()
    }
    root.present().unwrap();
    if DEBUG {
        std::fs::copy("/dev/shm/imgs/spec_all.png", "/dev/shm/back/spec_all.png").unwrap();
    }
    //
}

fn update() {
    create_dir_all(IMG_DIR_STR).unwrap();
    let freq: Vec<_> = (0..NCH).map(|i| i as f32 * 200_f32 / NCH as f32).collect();

    let cfg: Cfg = from_reader(File::open(CFG_FILE_NAME).unwrap()).unwrap();

    let stations = cfg.stations;
    let station_ant_map = BTreeMap::from_iter(
        stations
            .iter()
            .cloned()
            .enumerate()
            .map(|(aid, aname)| (aname, aid)),
    );

    let payload = read_payload(DATA_FILE_NAME);
    //let payload = read_payload("/dev/shm/back/dump.bin");

    let corr_prod = read_corr_prod(CORR_PROD_FILE_NAME);
    let corr_prod_map =
        BTreeMap::from_iter(corr_prod.iter().cloned().map(|x| ((x.a1, x.a2), x.nb)));

    for aid in &stations {
        plot_spec(aid, aid, &payload, &corr_prod_map, &station_ant_map, &freq);
    }

    for aid in &stations[1..] {
        plot_spec(
            "E01",
            aid,
            &payload,
            &corr_prod_map,
            &station_ant_map,
            &freq,
        );
    }

    /*
    for (i,aid1) in stations.iter().enumerate(){
        for aid2 in &stations[i..]{
            plot_spec(
                aid1,
                aid2,
                &payload,
                &corr_prod_map,
                &station_ant_map,
                &freq,
            );  
        }
    }*/

    plot_spec_all(&payload, &corr_prod_map, &station_ant_map, &freq);
}

fn main() {
    loop {
        update();
        let now = Local::now();
        println!("{:?}", now);


        std::thread::sleep(std::time::Duration::from_millis(100));
    }
}
