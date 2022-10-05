use newdaq::ctrl_msg::{self, CmdEnum};

use serde_yaml::to_writer;

fn main() {
    let msgs = vec![
        CmdEnum::HealthInfo(ctrl_msg::HealthInfo::default()),
        CmdEnum::SelfCheckStatus(ctrl_msg::SelfCheckStatus::default()),
        CmdEnum::WorkMode(ctrl_msg::WorkMode::default()),
        CmdEnum::FiveBoard(ctrl_msg::FiveBoard::new(3000)),
        CmdEnum::DataStatus(ctrl_msg::DataStatus::default()),
        CmdEnum::Stop(ctrl_msg::Stop::default()),
        CmdEnum::Shutdown(ctrl_msg::Shutdown::default()),
    ];

    let mut f = std::fs::File::create("sample.yaml").unwrap();
    to_writer(&mut f, &msgs).unwrap();
}
