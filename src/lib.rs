use std::{default::Default, path::PathBuf};
use chrono::{offset::Local, Date, Timelike, DateTime};
use sysinfo::{NetworkExt, NetworksExt, ProcessExt, System, SystemExt, DiskExt};

use num_complex::Complex;
pub const NCH: usize=8192;
pub const NCH_PER_PKT:usize=8192/8;
pub const NPORT_PER_BD:usize=8;
pub const PKT_LEN:usize=std::mem::size_of::<DataFrame>();

pub const NANTS:usize=40;
pub const NCORR:usize=NANTS*(NANTS+1)/2;

pub fn str2macarray(mac: &str) -> [u8; 6] {
    let mut result = [0_u8; 6];
    result
        .iter_mut()
        .zip(mac.split(':'))
        .for_each(|(x, y)| *x = u8::from_str_radix(y, 16).expect("not valid mac"));
    result
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct MetaData{
    _skip1: [u8; 16],
    pub bid1: u8,
    pub pid1: u8,
    pub bid2: u8,
    pub pid2: u8,
    pub pcnt: u8,
    _skip2: [u8;3],
    pub gcnt: u32,
    pub fcnt: u32,
}

impl Default for MetaData{
    fn default()->Self{
        MetaData { _skip1: [0_u8;16], bid1: 0, pid1: 0, bid2: 0, pid2: 0, pcnt: 0, _skip2: [0_u8;3], gcnt: 0, fcnt: 0 }
    }
}

#[repr(C)]
pub struct DataFrame{
    pub meta_data: MetaData
    , pub payload: [Complex<f32>; NCH_PER_PKT]
}


impl Default for DataFrame{
    fn default()->Self{
        DataFrame { meta_data: MetaData::default(), payload: [Complex::<f32>::default(); NCH_PER_PKT] }
    }
}

pub struct StorageMgr{
    pub pool: Vec<PathBuf>, 
    pub today: Date<Local>, 
    pub gbytes_per_day: usize,
}

impl StorageMgr{
    pub fn new(pool: Vec<PathBuf>, gbytes_per_day: usize)->Self{
        let today=Local::today().pred();
        Self { pool, today, gbytes_per_day }
    }

    pub fn get_out_dir(&mut self, now: DateTime<Local>)->PathBuf{        
        let today=now.date();
        if today!=self.today{
            println!("Yesterday is {}, today is {}", self.today, today);
            self.today=today;
                        
            let mut sys = System::new_all();
            sys.refresh_all();

            loop{
                if sys.disks().iter().any(|d| d.mount_point()==self.pool.first().unwrap() && {
                    let gbytes_to_write=(1.0-now.num_seconds_from_midnight() as f64/86400_f64)*(self.gbytes_per_day as f64);
                    let gbytes_available=d.available_space()/1024_u64.pow(3);
                    println!("{:?}: {} gbytes to write, {} gbytes available", d.mount_point(), gbytes_to_write, gbytes_available);
                    gbytes_available as f64 > gbytes_to_write
                }
                    
            ){
                    break;
                }
                else{
                    self.pool.rotate_left(1);
                }
            }
        }
        self.pool.first().unwrap().to_owned()
    }
}

