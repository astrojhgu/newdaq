console.log("hello")

function translate(s) {
    if (s == 'Writing') {
        return "正在写入"
    } else if (s == 'Spare') {
        return "--空盘--";
    } else if (s == 'Ejected') {
        return "未被挂载";
    } else if (s == 'Remounted') {
        return "重新挂载"
    }
    else {
        return s;
    }
}

//var timestamp;

async function update_state() {
    console.log("state updated");
    fetch("/data/last_msg_time.json?time=" + new Date()).then((response) => response.json())
        .then((data) => {
            timestamp = Date.parse(data["time"]);
            let dt = (Date.now() - timestamp) / 1000.0;
            dt = Math.round(dt * 10) / 10
            textContent = "最近一次状态更新时间:" + dt + " 秒之前";

            if (dt > 45) {
                textContent += "---警告：长时间未更新，检查仪器状态！";
                document.getElementById("timestamp").style.backgroundColor = "red";
            } else {
                document.getElementById("timestamp").style.backgroundColor = "green";
            }

            document.getElementById("timestamp").textContent = textContent;
            //document.getElementById("timestamp").textContent="Updated:   "+timestamp;
        });

    fetch("/data/last_data_time.json?time=" + new Date()).then((response) => response.json()).then((data) => {
        last_data_time = Date.parse(data['time']);
        let dt = (Date.now() - last_data_time) / 1000.0;
        dt = Math.round((dt - 3) * 10) / 10;
        textContent = "最近一次数据到达时间:" + dt + " 秒之前";
        if (dt > 10) {
            textContent += "---警告：长时间未更新，检查仪器状态！";
            document.getElementById("data_ts").style.backgroundColor = "red";
        } else {
            document.getElementById("data_ts").style.backgroundColor = "green";
        }
        document.getElementById("data_ts").textContent = textContent;
    })

    /*
    fetch("/data/temperature.json").then((response) => response.json())
        .then((data) => {
            temperature = data["temperature"];
            const temperature_list = document.getElementById("temperature_list");
            temperature_list.innerHTML = "";
            temperature_list.children = [];
            for (i = 0; i < 5; ++i) {
                const node = document.createElement("li");
                const textnode = document.createTextNode("板卡 " + i + ": " + temperature[i * 2] + " " + temperature[i * 2 + 1]);
                node.appendChild(textnode);
                temperature_list.appendChild(node);
            }
            const node = document.createElement("li");
            const textnode = document.createTextNode("K7 : " + temperature[10]);
            node.appendChild(textnode);
            temperature_list.appendChild(node);
        });
    */
    const promises = ["/data/temperature.json", "/data/state.json", "/data/check.json"].map(url => fetch(url).then(response => response.json()));
    const states = await Promise.all(promises);
    temperature = states[0]["temperature"]
    state = states[1]['sta'];
    check = states[2];
    
    const state_table = document.getElementById("tab_state");
    state_table.innerHTML = "";
    header = document.createElement("tr");
    for (x of ["板卡", "温度1", "温度2",
        "ADC PLL", "04821", "ADC CLK", "FPGA1 CLK",
        "光口1", "光口2", "光口3", "FPGA2 CLK",
        "K7光口1", "K7光口2", "板卡状态"]) {
        const h = document.createElement("th");
        h.appendChild(document.createTextNode(x));
        header.appendChild(h);
    }
    state_table.appendChild(header);

    for (i = 0; i < 5; ++i) {
        row = document.createElement("tr");
        data = [i, temperature[i * 2], temperature[i * 2 + 1],
            check["self_check_vu9p1"][4 * i + 0],
            check["self_check_vu9p1"][4 * i + 1],
            check["self_check_vu9p1"][4 * i + 2],
            check["self_check_vu9p1"][4 * i + 3],
            check["self_check_vu9p2"][i],
            check["self_check_vu9p2"][5 + i],
            check["self_check_vu9p2"][10 + i],
            check["self_check_vu9p2"][15 + i],
            check["self_check_k7"][2 * i],
            check["self_check_k7"][2 * i + 1],
            state[i] == 0 ? "正常" : "异常"
        ];

        var any_error=false;
        for(j=3;j<13;++j){
            if (data[j]!=1){
                any_error=true;
            }
        }

        for (d1 of data) {
            var d = document.createElement("td");
            d.appendChild(document.createTextNode(d1));
            row.appendChild(d);
        }
        if (state[i]==0 &&!any_error){
            header.style.backgroundColor="green";
            row.style.backgroundColor="green";
        }else{
            header.style.backgroundColor="red";
            row.style.backgroundColor="red";
        }

        state_table.appendChild(row);
    }


    fetch("/data/mode.json").then((response) => response.json())
        .then((data) => {
            if (data["mode"] == 3) {
                document.getElementById("state").textContent = "状态： 运行中"
                document.getElementById("state").style.backgroundColor = "green";
                //document.getElementById("div_tate").style.backgroundColor = "darkblue";
            } else {
                document.getElementById("state").textContent = "状态： 停止"
                document.getElementById("state").style.backgroundColor = "red";
                //document.getElementById("div_tate").style.backgroundColor = "red";

            }
        });

    fetch("/data/disk.json").then((response) => response.json())
        .then((data) => {
            disk_list = document.getElementById("disk_list");
            disk_list.innerHTML = "";
            for (dev in data) {
                const node = document.createElement("li")
                const textnode = document.createTextNode(data[dev]['slot'].split("/")[2].replaceAll('s', '盘位：') + " 盘符 " + dev + " :  " + translate(data[dev]['state']) + " " + data[dev]['occ']);
                node.appendChild(textnode)
                if (data[dev]['state'] == "Writing") {
                    node.style.backgroundColor = "green";
                    node.style.color = "white";
                } else if (data[dev]['state'] == "Ejected") {
                    node.style.backgroundColor = "gray";
                    node.style.color = "red";
                } else if (data[dev]['state'] == "Spare") {
                    node.style.backgroundColor = "yellow";
                    node.style.color = "green";
                } else if (data[dev]['state'] == "Remounted") {
                    node.style.backgroundColor = "cyan";
                    node.style.color = "magenta";
                }
                disk_list.appendChild(node);
                //console.log(data[dev])
            }
        });
    document.getElementById("current_time").innerHTML = (new Date()).toLocaleString('zh-CN');
}

var t = setInterval(update_state, 1000);

const ants = [
    "E01", "E02", "E03", "E04", "E05",
    "E06", "E07", "E08", "E09", "E10",
    "E11", "E12", "E13", "E14", "E15",
    "E16", "E17", "E18", "E19", "E20",
    "W01", "W02", "W03", "W04", "W05",
    "W06", "W07", "W08", "W09", "W10",
    "W11", "W12", "W13", "W14", "W15",
    "W16", "W17", "W18", "W19", "W20"];

for (let i = 0; i < 8; ++i) {
    let p = document.createElement("p");
    for (let j = 0; j < 5; ++j) {
        const n = i * 5 + j;
        let img = document.createElement("img");
        let ant = ants[n];
        img.src = "/data/imgs/" + ant + ant + "_ampl.png";
        img.width = 200;
        img.id = "spec_" + ant;
        p.appendChild(img);
    }
    document.getElementById("div_spec").appendChild(p);
    document.getElementById("div_spec").append(document.createElement("hr"));
}

for (let i = 0; i < 8; ++i) {
    let p = document.createElement("p");
    for (let j = 0; j < 5; ++j) {
        const n = i * 5 + j;
        let img = document.createElement("img");
        let ant = ants[n];
        img.src = "/data/imgs/" + "E01" + ant + "_ampl.png";
        img.width = 200;
        img.id = "corr_E01" + ant + "_ampl";
        p.appendChild(img);
    }
    document.getElementById("div_corr").appendChild(p);
    let p1 = document.createElement("p");
    for (let j = 0; j < 5; ++j) {
        const n = i * 5 + j;
        let img = document.createElement("img");
        let ant = ants[n];
        img.src = "/data/imgs/" + "E01" + ant + "_phase.png";
        img.width = 200;
        img.id = "corr_E01" + ant + "_phase";
        p1.appendChild(img);
    }
    document.getElementById("div_corr").appendChild(p1);

    document.getElementById("div_corr").append(document.createElement("hr"));
}

function update_imgs() {
    console.log("Images updated");
    document.getElementById("all_spec").src = "/data/imgs/spec_all.png?time=" + new Date();
    for (let ant of ants) {
        document.getElementById("spec_" + ant).src = "/data/imgs/" + ant + ant + "_ampl.png?time=" + new Date();
        document.getElementById("corr_E01" + ant + "_ampl").src = "/data/imgs/" + "E01" + ant + "_ampl.png?time=" + new Date();
        document.getElementById("corr_E01" + ant + "_phase").src = "/data/imgs/" + "E01" + ant + "_phase.png?time=" + new Date();
    }
}

setInterval(update_imgs, 10000);