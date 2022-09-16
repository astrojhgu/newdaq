use pnet::datalink::MacAddr;

pub fn mac2array(mac: &MacAddr) -> [u8; 6] {
    [mac.0, mac.1, mac.2, mac.3, mac.4, mac.5]
}

pub fn str2macarray(mac: &str) -> [u8; 6] {
    let mut result = [0_u8; 6];
    result
        .iter_mut()
        .zip(mac.split(':'))
        .for_each(|(x, y)| *x = u8::from_str_radix(y, 16).expect("not valid mac"));
    result
}
