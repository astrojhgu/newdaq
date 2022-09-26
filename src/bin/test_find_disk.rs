use std::path::PathBuf;

use chrono::Local;
use newdaq::StorageMgr;
fn main() {
    let pool: Vec<_> = vec!["/mnt/data1", "/mnt/data2"]
        .into_iter()
        .map(PathBuf::from)
        .collect();
    let mut storage = StorageMgr::new(pool, 1200);
    let now = Local::now();
    println!("{:?}", storage.get_out_dir(now));
    println!("{:?}", storage.get_out_dir(now));
}
