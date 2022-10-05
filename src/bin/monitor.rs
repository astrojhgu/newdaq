#![feature(proc_macro_hygiene, decl_macro)]

use std::{
    collections::BTreeMap,
    fs::{create_dir_all, File},
    io::Read,
    mem::size_of,
    path::PathBuf,
};

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
    loop {
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
            .filter(|x| x.is_ok())
            .map(|x: Result<CpRecord, _>| x.unwrap())
            .collect::<Vec<_>>();
        if result.len() == NCORR {
            break result;
        } else {
            println!("corr prod not ready");
            std::thread::sleep(std::time::Duration::from_millis(100));
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
    let caption = format!("{}-{}_ampl", fname, now.format("%m%d-%T"));
    let mut chart = ChartBuilder::on(&root)
        .caption(&caption, ("sans-serif", 50).into_font())
        .margin(5)
        .x_label_area_size(30)
        .y_label_area_size(30)
        .build_cartesian_2d(freq[0]..freq[freq.len() - 1], 50_f32..100_f32)
        .unwrap();
    chart.configure_mesh().draw().unwrap();

    chart
        .draw_series(LineSeries::new(
            freq.iter().cloned().zip(spec.iter().cloned()),
            &RED,
        ))
        .unwrap();
    root.present().unwrap();

    let out_img_name = img_out_dir.join(fname.clone() + "_phase.png");
    let root = BitMapBackend::new(&out_img_name, (800, 600)).into_drawing_area();
    root.fill(&WHITE).unwrap();
    let caption = format!("{}-{}_phase", fname, now.format("%m%d-%T"));
    let mut chart = ChartBuilder::on(&root)
        .caption(caption, ("sans-serif", 50).into_font())
        .margin(5)
        .x_label_area_size(30)
        .y_label_area_size(30)
        .build_cartesian_2d(freq[0]..freq[freq.len() - 1], -185_f32..185_f32)
        .unwrap();
    chart.configure_mesh().draw().unwrap();

    chart
        .draw_series(LineSeries::new(
            freq.iter().cloned().zip(phase.iter().cloned()),
            &BLUE,
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

    //println!("{:?}", spec);
    let now = Local::now();
    let img_out_dir = PathBuf::from(IMG_DIR_STR);
    
    let out_img_name = img_out_dir.join(fname.clone() + "_ampl.png");
    let title = format!("ampl-{}", now.format("%m%d-%T"));
    let root_ampl = BitMapBackend::new(&out_img_name, (2000, 2000)).into_drawing_area();
    root_ampl.fill(&WHITE).unwrap();
    
    let root_ampl=root_ampl.titled(&title, ("sans-serif", 100).into_font()).unwrap();
    //root_ampl.draw_text(&title, &("sans-serif", 250).into_text_style(&root_ampl), (200,200)).unwrap();

    let drawing_areas_ampl = root_ampl.split_evenly((NANTS, NANTS));

    let out_img_name = img_out_dir.join(fname.clone() + "_phase.png");

    let root_phase = BitMapBackend::new(&out_img_name, (2000, 2000)).into_drawing_area();

    root_phase.fill(&WHITE).unwrap();
    let title = format!("phase-{}",now.format("%m%d-%T"));
    let root_phase=root_phase.titled(&title, ("sans-serif", 100).into_font()).unwrap();
    //root_phase.draw_text(&title, &("sans-serif", 250).into_text_style(&root_ampl), (200,200)).unwrap();

    let drawing_areas_phase = root_phase.split_evenly((NANTS, NANTS));

    
    
    

    for (n, (root_ampl1, root_phase1)) in drawing_areas_ampl
        .iter()
        .zip(drawing_areas_phase.iter())
        .enumerate()
    {
        let i = n / NANTS;
        let j = n % NANTS;
        if i > j {
            continue;
        }

        let ncorr = corr_prod_map[&(i, j)];
        let data = &spec[ncorr * NCH..(ncorr + 1) * NCH];

        root_ampl1.fill(&WHITE).unwrap();
        
        let mut chart = ChartBuilder::on(&root_ampl1)
            .margin(5)
            .build_cartesian_2d(freq[0]..freq[freq.len() - 1], 50_f32..100_f32)
            .unwrap();
        //chart.configure_mesh().draw().unwrap();

        chart
            .draw_series(LineSeries::new(
                freq.iter().cloned().zip(data.iter().cloned()),
                &RED,
            ))
            .unwrap();

        root_phase1.fill(&WHITE).unwrap();
        let data = &phase[ncorr * NCH..(ncorr + 1) * NCH];
        let mut chart = ChartBuilder::on(&root_phase1)
            .margin(5)
            .build_cartesian_2d(freq[0]..freq[freq.len() - 1], -180_f32..180_f32)
            .unwrap();
        //chart.configure_mesh().draw().unwrap();

        chart
            .draw_series(LineSeries::new(
                freq.iter().cloned().zip(data.iter().cloned()).skip(NCH/4+100).take(NCH*3/4-200),
                &BLUE,
            ))
            .unwrap();
        //root1.present().unwrap()
    }

    root_ampl.present().unwrap();
    root_phase.present().unwrap();
}

fn power_all(
    payload: &[Complex32],
    corr_prod_map: &BTreeMap<(usize, usize), usize>,
    _station_ant_map: &BTreeMap<String, usize>,
    _freq: &[f32],
) {
    let now = Local::now();
    let caption = format!("Power {}", now.format("%m%d-%T"));
    let root = BitMapBackend::new("/dev/shm/imgs/power.png", (800, 600)).into_drawing_area();
    root.fill(&WHITE).unwrap();
    let mut chart = ChartBuilder::on(&root)
        .caption(&caption, ("sans-serif", 20))
        .margin(5)
        .top_x_label_area_size(40)
        .y_label_area_size(40)
        .build_cartesian_2d(0i32..NANTS as i32, NANTS as i32..0_i32)
        .unwrap();

    chart
        .configure_mesh()
        .x_labels(NANTS)
        .y_labels(NANTS)
        .max_light_lines(4)
        .disable_x_mesh()
        .disable_y_mesh()
        .label_style(("sans-serif", 12))
        .draw()
        .unwrap();

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

    for i in 0..NANTS {
        for j in 0..i {
            matrix[i][j] = vmax;
        }
    }

    chart
        .draw_series(
            matrix
                .iter()
                .zip(0..)
                .map(|(l, y)| l.iter().zip(0..).map(move |(v, x)| (x as i32, y as i32, v)))
                .flatten()
                .map(|(x, y, v)| {
                    Rectangle::new(
                        [(x, y), (x + 1, y + 1)],
                        HSLColor(0.45, 0.7, (*v / vmax) as f64).filled(),
                    )
                }),
        )
        .unwrap();

    root.present().unwrap();
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

    power_all(&payload, &corr_prod_map, &station_ant_map, &freq);
    plot_spec_all(&payload, &corr_prod_map, &station_ant_map, &freq);
}

fn main() {
    loop {
        update();
        let now = Local::now();
        println!("{:?}", now);
        std::thread::sleep(std::time::Duration::from_secs(10));
    }
}
