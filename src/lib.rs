#![feature(new_uninit)]

use chrono::{offset::Local, Date, DateTime, Timelike};
use lockfile::Lockfile;
use serde::{Deserialize, Serialize};
use std::{default::Default, fs::File, path::PathBuf};
use sysinfo::{DiskExt, System, SystemExt};

use num_complex::Complex;

pub mod ctrl_msg;

pub const NCH: usize = 8192;
pub const NCH_PER_PKT: usize = 8192 / 8;
pub const NPORT_PER_BD: usize = 8;
pub const PKT_LEN: usize = std::mem::size_of::<DataFrame>();
pub const NBOARD: usize = 5;
pub const NANTS: usize = 40;
pub const NCORR: usize = NANTS * (NANTS + 1) / 2;

pub fn str2macarray(mac: &str) -> [u8; 6] {
    let mut result = [0_u8; 6];
    result
        .iter_mut()
        .zip(mac.split(':'))
        .for_each(|(x, y)| *x = u8::from_str_radix(y, 16).expect("not valid mac"));
    result
}

#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct Cfg {
    pub dev: String,
    pub out_dir: Vec<std::path::PathBuf>,
    pub gbytes_per_day: usize,
    pub stations: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct TimeStamp {
    pub time: String,
}

#[derive(Clone, Copy, Default)]
#[repr(C)]
pub struct MetaData {
    _skip1: [u8; 16],
    pub bid1: u8,
    pub pid1: u8,
    pub bid2: u8,
    pub pid2: u8,
    pub pcnt: u8,
    _skip2: [u8; 3],
    pub gcnt: u32,
    pub fcnt: u32,
}

#[repr(C)]
pub struct DataFrame {
    pub meta_data: MetaData,
    pub payload: [Complex<f32>; NCH_PER_PKT],
}

impl Default for DataFrame {
    fn default() -> Self {
        DataFrame {
            meta_data: MetaData::default(),
            payload: [Complex::<f32>::default(); NCH_PER_PKT],
        }
    }
}

impl DataFrame {
    pub fn from_raw(src: &[u8]) -> Self {
        unsafe { std::ptr::read(src.as_ptr() as *const Self) }
    }
}

#[derive(Debug, Clone)]
#[repr(C)]
pub struct RawDataFrame {
    //time domain data
    pub cnt: u64,
    pub payload: [u16; 2048],
}

impl Default for RawDataFrame {
    fn default() -> Self {
        Self {
            cnt: 0,
            payload: [0; 2048],
        }
    }
}

impl RawDataFrame {
    pub fn from_raw(x: &[u8]) -> Self {
        assert!(x.len() == 4104);
        unsafe { std::ptr::read_unaligned(x.as_ptr() as *const Self) }
    }
}

pub struct StorageMgr {
    pub pool: Vec<PathBuf>,
    pub today: Date<Local>,
    pub gbytes_per_day: usize,
    pub lock: Option<Lockfile>,
}

impl StorageMgr {
    pub fn new(pool: Vec<PathBuf>, gbytes_per_day: usize) -> Self {
        let today = Local::today().pred();
        Self {
            pool,
            today,
            gbytes_per_day,
            lock: None,
        }
    }

    pub fn get_out_dir(&mut self, now: DateTime<Local>) -> PathBuf {
        let today = now.date();
        if today != self.today {
            println!("Yesterday is {}, today is {}", self.today, today);
            self.today = today;

            loop {
                let mut sys = System::new_all();
                sys.refresh_all();
                if sys.disks().iter().any(|d| {
                    //println!("{:?}", d.name());
                    d.mount_point() == self.pool.first().unwrap() && {
                        let gbytes_to_write = (1.0
                            - now.num_seconds_from_midnight() as f64 / 86400_f64)
                            * (self.gbytes_per_day as f64);
                        let gbytes_available = d.available_space() / 1024_u64.pow(3);
                        println!(
                            "{:?}: {} gbytes to write, {} gbytes available",
                            d.mount_point(),
                            gbytes_to_write,
                            gbytes_available
                        );
                        if gbytes_available as f64 > gbytes_to_write {
                            if std::fs::remove_file(self.pool.first().unwrap().join("running"))
                                .is_ok()
                            {
                                println!("removed lock file");
                            }
                            self.lock = Some(
                                Lockfile::create(self.pool.first().unwrap().join("running"))
                                    .unwrap(),
                            );
                            true
                        } else {
                            if File::create(self.pool.first().unwrap().join("done")).is_ok() {}
                            false
                        }
                    }
                }) {
                    break;
                } else {
                    self.pool.rotate_left(1);
                }
                println!("checking next candidate disk");
                std::thread::sleep(std::time::Duration::from_millis(1000));
            }
        }
        self.pool.first().unwrap().to_owned()
    }
}
