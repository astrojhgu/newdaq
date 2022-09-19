use std::default::Default;

use pnet::datalink::MacAddr;
use num_complex::Complex;
pub const NCH: usize=8192;
pub const NCH_PER_PKT:usize=8192/8;
pub const NPORT_PER_BD:usize=8;
pub const PKT_LEN:usize=std::mem::size_of::<DataFrame>();

pub const NANTS:usize=40;
pub const NCORR:usize=NANTS*(NANTS+1)/2;

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
