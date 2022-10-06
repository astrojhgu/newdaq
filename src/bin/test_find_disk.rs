fn main() {
    let _f = std::fs::File::create("/mnt/sdb/a.dat").unwrap();
    std::thread::sleep(std::time::Duration::from_secs(10));
}
