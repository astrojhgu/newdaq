use std::path::PathBuf;

use newdaq::StorageMgr;
use sysinfo::{NetworkExt, NetworksExt, ProcessExt, System, SystemExt, DiskExt};
use chrono::{Local};
fn main(){
    let pool:Vec<_>=vec!["/mnt/data1", "/mnt/data2"].into_iter().map(|d| PathBuf::from(d)).collect();
    let mut storage=StorageMgr::new(pool, 1200);
    let now=Local::now();
    println!("{:?}", storage.get_out_dir(now));
    println!("{:?}", storage.get_out_dir(now));
}