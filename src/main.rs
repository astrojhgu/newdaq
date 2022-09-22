use std::slice::SliceIndex;

use clap::{Arg, Command};

use etherparse::{
    Ethernet2HeaderSlice, InternetSlice, LinkSlice, SlicedPacket, TcpOptionElement,
    TcpOptionReadError, TransportSlice,
};

use newdaq::{str2macarray, DataFrame, PKT_LEN, MetaData};
const SPEC_PER_SEC: usize = 122071;

fn main() {

}
