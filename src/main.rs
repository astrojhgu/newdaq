use newdaq::ctrl_msg::{self, CmdEnum, Command, GB40};

use serde_yaml::to_writer;

fn main() {
    let mut gb40 = GB40::default();
    let mut gb40_1 = GB40::default();

    gb40.enable_board(0, 0, [192, 168, 1, 10], 8888, [192, 168, 1, 20], 8888);
    gb40.enable_board(1, 0, [192, 168, 1, 11], 8888, [192, 168, 1, 20], 8888);
    gb40.enable_board(2, 0, [192, 168, 1, 12], 8888, [192, 168, 1, 20], 8888);
    gb40.enable_board(3, 0, [192, 168, 1, 13], 8888, [192, 168, 1, 20], 8888);

    let msgs = vec![
        CmdEnum::HealthInfo(ctrl_msg::HealthInfo::default()),
        CmdEnum::SelfCheckStatus(ctrl_msg::SelfCheckStatus::default()),
        CmdEnum::WorkMode(ctrl_msg::WorkMode::default()),
        CmdEnum::FiveBoard(ctrl_msg::FiveBoard::new(3000)),
        CmdEnum::DataStatus(ctrl_msg::DataStatus::default()),
        CmdEnum::Stop(ctrl_msg::Stop::default()),
        CmdEnum::Shutdown(ctrl_msg::Shutdown::default()),
        CmdEnum::GB40(gb40),
    ];

    let mut data = vec![0_u8; 72];

    println!("{}", gb40.fill_data(&mut data));

    println!("{:?}", &data);

    gb40_1.from_data(&data);

    assert_eq!(gb40, gb40_1);

    let mut f = std::fs::File::create("sample.yaml").unwrap();
    to_writer(&mut f, &msgs).unwrap();
}
