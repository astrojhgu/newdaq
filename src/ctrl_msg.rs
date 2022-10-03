use crate::NBOARD;
use packed_struct::{prelude::*, types::bits::ByteArray};
use std::convert::Into;
use std::default::Default;

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, PackedStruct, PartialEq, Eq, Debug)]
#[packed_struct(endian = "lsb")]
pub struct CommandFrame {
    pub frame_head: u32,
    pub info_func: u8,
    pub packet_len: u32,
    pub frame_tale: u32,
    pub data: [u8; 100],
}

impl Default for CommandFrame {
    fn default() -> Self {
        CommandFrame {
            frame_head: 0x5555aaaa,
            info_func: 0,
            packet_len: 0,
            frame_tale: 0xaaaabbbb,
            data: [0_u8; 100],
        }
    }
}

impl CommandFrame {
    pub fn from_msg(msg: &dyn Command) -> Self {
        let mut data = [0_u8; 100];
        let sz = msg.fill_data(&mut data);
        Self {
            frame_head: 0x5555aaaa,
            info_func: msg.cmd_type() as u8,
            packet_len: sz as u32,
            frame_tale: 0xaaaabbbb,
            data,
        }
    }

    pub fn get_cmd(&self) -> Box<dyn Command> {
        let cmd_type = CmdType::from_u8(self.info_func);
        use CmdType::*;

        match cmd_type {
            DoubleBoardSet => Box::new(
                DoubleBoard::unpack_from_slice(
                    &self.data[..<DoubleBoard as PackedStruct>::ByteArray::len()],
                )
                .unwrap(),
            ),
            FiveBoardSet => Box::new(
                FiveBoard::unpack_from_slice(
                    &self.data[..<FiveBoard as PackedStruct>::ByteArray::len()],
                )
                .unwrap(),
            ),
            ModeFor40GB => Box::new(
                GB40::unpack_from_slice(&self.data[..<GB40 as PackedStruct>::ByteArray::len()])
                    .unwrap(),
            ),
            QueryDataStatus => Box::new(
                DataStatus::unpack_from_slice(
                    &self.data[..<DataStatus as PackedStruct>::ByteArray::len()],
                )
                .unwrap(),
            ),
            QueryHealthInfo => Box::new(
                HealthInfo::unpack_from_slice(
                    &self.data[..<HealthInfo as PackedStruct>::ByteArray::len()],
                )
                .unwrap(),
            ),
            QuerySelfCheckStatus => Box::new(
                SelfCheckStatus::unpack_from_slice(
                    &self.data[..<SelfCheckStatus as PackedStruct>::ByteArray::len()],
                )
                .unwrap(),
            ),
            QueryWorkMode => Box::new(
                WorkMode::unpack_from_slice(
                    &self.data[..<WorkMode as PackedStruct>::ByteArray::len()],
                )
                .unwrap(),
            ),
            ReferenceSet => Box::new(
                Reference::unpack_from_slice(
                    &self.data[..<Reference as PackedStruct>::ByteArray::len()],
                )
                .unwrap(),
            ),
            ShutDownNow => Box::new(
                Shutdown::unpack_from_slice(
                    &self.data[..<Shutdown as PackedStruct>::ByteArray::len()],
                )
                .unwrap(),
            ),
            SingleBoardSet => Box::new(
                SingleBoard::unpack_from_slice(
                    &self.data[..<SingleBoard as PackedStruct>::ByteArray::len()],
                )
                .unwrap(),
            ),
            Stop => Box::new(
                crate::ctrl_msg::Stop::unpack_from_slice(
                    &self.data[..<crate::ctrl_msg::Stop as PackedStruct>::ByteArray::len()],
                )
                .unwrap(),
            ),
            TriggerSet => Box::new(
                Trigger::unpack_from_slice(
                    &self.data[..<Trigger as PackedStruct>::ByteArray::len()],
                )
                .unwrap(),
            ),
            _ => panic!(),
        }
    }
}

pub enum CmdType {
    Stop = 0x01,
    SingleBoardSet,       // 1块板卡模式启动 2
    DoubleBoardSet,       // 2块板卡模式启动 3
    FiveBoardSet,         // 5块板卡模式启动 4
    QuerySelfCheckStatus, // 自检结果查询  5
    QueryHealthInfo,      // 健康（板卡温度）信息查询 6
    QueryWorkMode,        // 工作模式查询，具体模式见 enum Work_Mode 中定义 7
    ModeFor40GB,          // 40GB模式启动8
    TriggerSet,           // 内外触发设置9
    ReferenceSet,         // 内外参考设置10
    ShutDownNow,          // 关机11
    QueryDataStatus, // 数据状态是否正常查询（只有当设备工作在相关模式，且正处在工作过程中时，此查询结果有效）12
    Unknown,
}

impl CmdType {
    pub fn from_u8(x: u8) -> Self {
        match x {
            0x01 => CmdType::Stop,
            0x02 => CmdType::SingleBoardSet,
            0x03 => CmdType::DoubleBoardSet,
            0x04 => CmdType::FiveBoardSet,
            0x05 => CmdType::QuerySelfCheckStatus,
            0x06 => CmdType::QueryHealthInfo,
            0x07 => CmdType::QueryWorkMode,
            0x08 => CmdType::ModeFor40GB,
            0x09 => CmdType::TriggerSet,
            0x0a => CmdType::ReferenceSet,
            0x0b => CmdType::ShutDownNow,
            0x0c => CmdType::QueryDataStatus,
            _ => CmdType::Unknown,
        }
    }
}

pub enum BoardID {
    B0 = 1,
    B1,
    B2,
    B3,
    B4,
}

pub trait Command {
    fn cmd_type(&self) -> CmdType;
    fn cmd_string(&self) -> String;
    fn fill_data(&self, _: &mut [u8]) -> usize;
    #[allow(clippy::wrong_self_convention)]
    fn from_data(&mut self, _: &[u8]);
    fn to_enum(&self)->CmdEnum;
}
/////////////////////////////////////////////////////////////////////
#[derive(Clone, Copy, PackedStruct, PartialEq, Eq, Debug, Serialize, Deserialize, Default)]
#[packed_struct(endian = "lsb")]
pub struct SingleBoard {
    pub board_num: u8,
    pub resolution: u32,
}

impl Command for SingleBoard {
    fn cmd_type(&self) -> CmdType {
        CmdType::SingleBoardSet
    }

    fn cmd_string(&self) -> String {
        format!("SingleBoard: {} {}", self.board_num, self.resolution)
    }

    fn fill_data(&self, d: &mut [u8]) -> usize {
        let sz = <Self as PackedStruct>::ByteArray::len();
        self.pack_to_slice(&mut d[..sz]).unwrap();
        sz
    }

    fn from_data(&mut self, d: &[u8]) {
        let sz = <Self as PackedStruct>::ByteArray::len();
        *self = Self::unpack_from_slice(&d[..sz]).unwrap();
    }

    fn to_enum(&self)->CmdEnum {
        CmdEnum::SingleBoard(*self)
    }
}

#[derive(Clone, Copy, PackedStruct, PartialEq, Eq, Debug, Serialize, Deserialize, Default)]
#[packed_struct(endian = "lsb")]
pub struct DoubleBoard {
    pub board_num1: u8,  // 板卡号1 范围（1~5）
    pub board_num2: u8,  // 板卡号2 范围（1~5）板卡号2 应大于 板卡号1
    pub resolution: u32, // 分辨率 以ms为单位
}

impl Command for DoubleBoard {
    fn cmd_type(&self) -> CmdType {
        CmdType::DoubleBoardSet
    }

    fn cmd_string(&self) -> String {
        format!(
            "DoubleBoard {} {} {}",
            self.board_num1 as u32, self.board_num2 as u32, self.resolution
        )
    }

    fn fill_data(&self, d: &mut [u8]) -> usize {
        let sz = <Self as PackedStruct>::ByteArray::len();
        self.pack_to_slice(&mut d[..sz]).unwrap();
        sz
    }

    fn from_data(&mut self, d: &[u8]) {
        let sz = <Self as PackedStruct>::ByteArray::len();
        *self = Self::unpack_from_slice(&d[..sz]).unwrap();
    }

    fn to_enum(&self)->CmdEnum {
        CmdEnum::DoubleBoard(*self)
    }
}

#[derive(Clone, Copy, PackedStruct, PartialEq, Eq, Debug, Serialize, Deserialize, Default)]
#[packed_struct(endian = "lsb")]
pub struct FiveBoard {
    pub resolution: u32, // 分辨率 以ms为单位
}

impl FiveBoard {
    pub fn new(resolution: u32) -> Self {
        Self { resolution }
    }
}

impl Command for FiveBoard {
    fn cmd_type(&self) -> CmdType {
        CmdType::FiveBoardSet
    }

    fn cmd_string(&self) -> String {
        format!("FiveBoard {}", self.resolution)
    }

    fn fill_data(&self, d: &mut [u8]) -> usize {
        let sz = <Self as PackedStruct>::ByteArray::len();
        self.pack_to_slice(&mut d[..sz]).unwrap();
        sz
    }

    fn from_data(&mut self, d: &[u8]) {
        let sz = <Self as PackedStruct>::ByteArray::len();
        *self = Self::unpack_from_slice(&d[..sz]).unwrap();
    }

    fn to_enum(&self)->CmdEnum {
        CmdEnum::FiveBoard(*self)
    }
}

#[derive(Clone, Copy, PackedStruct, PartialEq, Eq, Debug, Serialize, Deserialize, Default)]
#[packed_struct(endian = "lsb")]
pub struct SelfCheckStatus {
    pub self_check_vu9p1: [u8; 20],
    pub self_check_vu9p2: [u8; 20],
    pub self_check_k7: [u8; 10],
}

impl Command for SelfCheckStatus {
    fn cmd_type(&self) -> CmdType {
        CmdType::QuerySelfCheckStatus
    }

    fn cmd_string(&self) -> String {
        let mut result = "SelfCheckStatus:\n".to_string();
        for i in 0..NBOARD {
            let s = format!(
                "{} {} {} {} | {} {} {} {} | {} {} \n",
                self.self_check_vu9p1[i * 4],
                self.self_check_vu9p1[i * 4 + 1],
                self.self_check_vu9p1[i * 4 + 2],
                self.self_check_vu9p1[i * 4 + 3],
                self.self_check_vu9p2[i * 4],
                self.self_check_vu9p2[i * 4 + 1],
                self.self_check_vu9p2[i * 4 + 2],
                self.self_check_vu9p2[i * 4 + 3],
                self.self_check_k7[i * 2],
                self.self_check_k7[i * 2 + 1],
            );
            result += &s;
        }
        result
    }

    fn fill_data(&self, d: &mut [u8]) -> usize {
        let sz = <Self as PackedStruct>::ByteArray::len();
        self.pack_to_slice(&mut d[..sz]).unwrap();
        sz
    }

    fn from_data(&mut self, d: &[u8]) {
        let sz = <Self as PackedStruct>::ByteArray::len();
        *self = Self::unpack_from_slice(&d[..sz]).unwrap();
    }

    fn to_enum(&self)->CmdEnum {
        CmdEnum::SelfCheckStatus(*self)
    }
}

#[derive(Clone, Copy, PackedStruct, PartialEq, Eq, Debug, Serialize, Deserialize, Default)]
#[packed_struct(endian = "lsb")]
pub struct HealthInfo {
    temperature: [u32; 11],
}

impl Command for HealthInfo {
    fn cmd_type(&self) -> CmdType {
        CmdType::QueryHealthInfo
    }

    fn cmd_string(&self) -> String {
        format!(
            "Temperature \n{} {}\n{} {}\n{} {}\n{} {}\n{} {}\n{}\n",
            self.temperature[0],
            self.temperature[1],
            self.temperature[2],
            self.temperature[3],
            self.temperature[4],
            self.temperature[5],
            self.temperature[6],
            self.temperature[7],
            self.temperature[8],
            self.temperature[9],
            self.temperature[10]
        )
    }

    fn fill_data(&self, d: &mut [u8]) -> usize {
        let sz = <Self as PackedStruct>::ByteArray::len();
        self.pack_to_slice(&mut d[..sz]).unwrap();
        sz
    }

    fn from_data(&mut self, d: &[u8]) {
        let sz = <Self as PackedStruct>::ByteArray::len();
        *self = Self::unpack_from_slice(&d[..sz]).unwrap();
    }

    fn to_enum(&self)->CmdEnum {
        CmdEnum::HealthInfo(*self)
    }
}

#[derive(Clone, Copy, PackedStruct, PartialEq, Eq, Debug, Serialize, Deserialize, Default)]
#[packed_struct(endian = "lsb")]
pub struct GB40 {
    pub board_num1: u8, // 第1块卡是否工作 非0有效
    pub channel1: u8,   // 若board_num1为有效，该字段为板卡1具体工作的通道，配置范围为 0~7

    pub board_num2: u8, // 第2块卡是否工作 非0有效
    pub channel2: u8,   // 若board_num2为有效，该字段为板卡2具体工作的通道，配置范围为 0~7

    pub board_num3: u8, // 第3块卡是否工作 非0有效
    pub channel3: u8,   // 若board_num3为有效，该字段为板卡3具体工作的通道，配置范围为 0~7

    pub board_num4: u8, // 第5块卡是否工作 非0有效
    pub channel4: u8,   // 若board_num4为有效，该字段为板卡5具体工作的通道，配置范围为 0~7

    pub local_ip1: [u8; 4], // 第1块卡对应的本地IP配置，local_ip1[0]非0有效
    pub local_port1: u16, // 第1块卡对应的本地端口配置，当local_ip1[0]有效时，该字段也会进行同步配置

    pub local_ip2: [u8; 4], // 第2块卡对应的本地IP配置，local_ip2[0]非0有效
    pub local_port2: u16, // 第2块卡对应的本地端口配置，当local_ip2[0]有效时，该字段也会进行同步配置

    pub local_ip3: [u8; 4], // 第3块卡对应的本地IP配置，local_ip3[0]非0有效
    pub local_port3: u16, // 第3块卡对应的本地端口配置，当local_ip3[0]有效时，该字段也会进行同步配置

    pub local_ip4: [u8; 4], // 第5块卡对应的本地IP配置，local_ip4[0]非0有效
    pub local_port4: u16, // 第5块卡对应的本地端口配置，当local_ip4[0]有效时，该字段也会进行同步配置

    pub optical_ip1: [u8; 4], // 第1块卡对应的对端IP配置，optical_ip1[0]非0有效
    pub optical_port1: u16, // 第1块卡对应的对端端口配置，当optical_ip1[0]有效时，该字段也会进行同步配置

    pub optical_ip2: [u8; 4], // 第2块卡对应的对端IP配置，optical_ip2[0]非0有效
    pub optical_port2: u16, // 第2块卡对应的对端端口配置，当optical_ip2[0]有效时，该字段也会进行同步配置

    pub optical_ip3: [u8; 4], // 第3块卡对应的对端IP配置，optical_ip3[0]非0有效
    pub optical_port3: u16, // 第3块卡对应的对端端口配置，当optical_ip3[0]有效时，该字段也会进行同步配置

    pub optical_ip4: [u8; 4], // 第5块卡对应的对端IP配置，optical_ip4[0]非0有效
    pub optical_port4: u16, // 第5块卡对应的对端端口配置，当optical_ip4[0]有效时，该字段也会进行同步配置
}

impl Command for GB40 {
    fn cmd_type(&self) -> CmdType {
        CmdType::ModeFor40GB
    }

    fn cmd_string(&self) -> String {
        "GB40".to_string()
    }

    fn fill_data(&self, d: &mut [u8]) -> usize {
        let sz = <Self as PackedStruct>::ByteArray::len();
        self.pack_to_slice(&mut d[..sz]).unwrap();
        sz
    }

    fn from_data(&mut self, d: &[u8]) {
        let sz = <Self as PackedStruct>::ByteArray::len();
        *self = Self::unpack_from_slice(&d[..sz]).unwrap();
    }

    fn to_enum(&self)->CmdEnum {
        CmdEnum::GB40(*self)
    }
}

#[derive(Clone, Copy, PackedStruct, PartialEq, Eq, Debug, Serialize, Deserialize, Default)]
#[packed_struct(endian = "lsb")]
pub struct Trigger {
    pub trigger_value: u8,
}

impl Command for Trigger {
    fn cmd_type(&self) -> CmdType {
        CmdType::TriggerSet
    }

    fn cmd_string(&self) -> String {
        "Trigger".to_string()
    }

    fn fill_data(&self, d: &mut [u8]) -> usize {
        let sz = <Self as PackedStruct>::ByteArray::len();
        self.pack_to_slice(&mut d[..sz]).unwrap();
        sz
    }

    fn from_data(&mut self, d: &[u8]) {
        let sz = <Self as PackedStruct>::ByteArray::len();
        *self = Self::unpack_from_slice(&d[..sz]).unwrap();
    }

    fn to_enum(&self)->CmdEnum {
        CmdEnum::Trigger(*self)
    }
}

#[derive(Clone, Copy, PackedStruct, PartialEq, Eq, Debug, Serialize, Deserialize, Default)]
#[packed_struct(endian = "lsb")]
pub struct Reference {
    pub reference_value: u8,
}

impl Command for Reference {
    fn cmd_type(&self) -> CmdType {
        CmdType::ReferenceSet
    }

    fn cmd_string(&self) -> String {
        format!("Reference: {}", self.reference_value)
    }

    fn fill_data(&self, d: &mut [u8]) -> usize {
        let sz = <Self as PackedStruct>::ByteArray::len();
        self.pack_to_slice(&mut d[..sz]).unwrap();
        sz
    }

    fn from_data(&mut self, d: &[u8]) {
        let sz = <Self as PackedStruct>::ByteArray::len();
        *self = Self::unpack_from_slice(&d[..sz]).unwrap();
    }

    fn to_enum(&self)->CmdEnum {
        CmdEnum::Reference(*self)
    }
}

#[derive(Clone, Copy, PackedStruct, PartialEq, Eq, Debug, Serialize, Deserialize, Default)]
#[packed_struct(endian = "lsb")]
pub struct WorkMode {
    pub mode: u32,
}

impl Command for WorkMode {
    fn cmd_type(&self) -> CmdType {
        CmdType::QueryWorkMode
    }

    fn cmd_string(&self) -> String {
        format!("WorkMode: {}", self.mode)
    }

    fn fill_data(&self, d: &mut [u8]) -> usize {
        let sz = <Self as PackedStruct>::ByteArray::len();
        self.pack_to_slice(&mut d[..sz]).unwrap();
        sz
    }

    fn from_data(&mut self, d: &[u8]) {
        let sz = <Self as PackedStruct>::ByteArray::len();
        *self = Self::unpack_from_slice(&d[..sz]).unwrap();
    }

    fn to_enum(&self)->CmdEnum {
        CmdEnum::WorkMode(*self)
    }
}

#[derive(Clone, Copy, PackedStruct, PartialEq, Eq, Debug, Serialize, Deserialize, Default)]
#[packed_struct(endian = "lsb")]
pub struct DataStatus {
    pub sta: [u32; 5],
}

impl Command for DataStatus {
    fn cmd_type(&self) -> CmdType {
        CmdType::QueryDataStatus
    }

    fn cmd_string(&self) -> String {
        format!(
            "DataStatus: {} {} {} {} {}\n",
            self.sta[0], self.sta[1], self.sta[2], self.sta[3], self.sta[4]
        )
    }

    fn fill_data(&self, d: &mut [u8]) -> usize {
        let sz = <Self as PackedStruct>::ByteArray::len();
        self.pack_to_slice(&mut d[..sz]).unwrap();
        sz
    }

    fn from_data(&mut self, d: &[u8]) {
        let sz = <Self as PackedStruct>::ByteArray::len();
        *self = Self::unpack_from_slice(&d[..sz]).unwrap();
    }

    fn to_enum(&self)->CmdEnum {
        CmdEnum::DataStatus(*self)
    }
}

#[derive(Clone, Copy, PackedStruct, PartialEq, Eq, Debug, Serialize, Deserialize, Default)]
#[packed_struct(endian = "lsb")]
pub struct Shutdown {
    pub _x: u8,
}

impl Command for Shutdown {
    fn cmd_type(&self) -> CmdType {
        CmdType::ShutDownNow
    }

    fn cmd_string(&self) -> String {
        "Shutdown".to_string()
    }

    fn fill_data(&self, d: &mut [u8]) -> usize {
        let sz = <Self as PackedStruct>::ByteArray::len();
        self.pack_to_slice(&mut d[..sz]).unwrap();
        sz
    }

    fn from_data(&mut self, d: &[u8]) {
        let sz = <Self as PackedStruct>::ByteArray::len();
        *self = Self::unpack_from_slice(&d[..sz]).unwrap();
    }
    fn to_enum(&self)->CmdEnum {
        CmdEnum::Shutdown(*self)
    }
}

#[derive(Clone, Copy, PackedStruct, PartialEq, Eq, Debug, Serialize, Deserialize, Default)]
#[packed_struct(endian = "lsb")]
pub struct Stop {
    pub _x: u8,
}

impl Command for Stop {
    fn cmd_type(&self) -> CmdType {
        CmdType::Stop
    }

    fn cmd_string(&self) -> String {
        "Stop".to_string()
    }

    fn fill_data(&self, d: &mut [u8]) -> usize {
        let sz = <Self as PackedStruct>::ByteArray::len();
        self.pack_to_slice(&mut d[..sz]).unwrap();
        sz
    }

    fn from_data(&mut self, d: &[u8]) {
        let sz = <Self as PackedStruct>::ByteArray::len();
        *self = Self::unpack_from_slice(&d[..sz]).unwrap();
    }

    fn to_enum(&self)->CmdEnum {
        CmdEnum::Stop(*self)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum CmdEnum {
    SingleBoard(SingleBoard),
    DoubleBoard(DoubleBoard),
    FiveBoard(FiveBoard),
    SelfCheckStatus(SelfCheckStatus),
    HealthInfo(HealthInfo),
    GB40(GB40),
    Trigger(Trigger),
    Reference(Reference),
    WorkMode(WorkMode),
    DataStatus(DataStatus),
    Shutdown(Shutdown),
    Stop(Stop),
}

impl CmdEnum {
    pub fn get_cmd(&self) -> Box<dyn Command> {
        match self {
            CmdEnum::SingleBoard(a) => Box::new(*a),
            CmdEnum::DoubleBoard(a) => Box::new(*a),
            CmdEnum::FiveBoard(a) => Box::new(*a),
            CmdEnum::SelfCheckStatus(a) => Box::new(*a),
            CmdEnum::HealthInfo(a) => Box::new(*a),
            CmdEnum::GB40(a) => Box::new(*a),
            CmdEnum::Trigger(a) => Box::new(*a),
            CmdEnum::Reference(a) => Box::new(*a),
            CmdEnum::WorkMode(a) => Box::new(*a),
            CmdEnum::DataStatus(a) => Box::new(*a),
            CmdEnum::Shutdown(a) => Box::new(*a),
            CmdEnum::Stop(a) => Box::new(*a),
        }
    }
}
