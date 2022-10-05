console.log("hello")

fetch("/data/mode.json").then((response) => response.json())
.then((data) => {
    if (data["mode"]==3){
        document.getElementById("status").textContent="状态： 运行中"
    }else{
        document.getElementById("status").textContent="状态： 停止"
    }
});

fetch("/data/temperature.json").then((response) => response.json())
.then((data) => {
    temperature=data["temperature"];
    const temperature_list=document.getElementById("temperature_list");
    for(i=0;i<5;++i){
        const node=document.createElement("li");
        const textnode=document.createTextNode("板卡 "+i+" "+temperature[i*2]+" "+temperature[i*2+1]);
        node.appendChild(textnode);
        temperature_list.appendChild(node);
    }
});

var timestamp;


fetch("/data/disk.json").then((response)=> response.json())
.then((data)=>{
    disk_list=document.getElementById("disk_list");
    for (dev in data){
        const node=document.createElement("li")
        const textnode=document.createTextNode(data[dev]['slot']+" "+dev+" :  "+data[dev]['state']+" "+data[dev]['occ']);
        node.appendChild(textnode)
        disk_list.appendChild(node);
        console.log(data[dev])
    }
});

var t=setInterval(function(){
    fetch("/data/last_msg_time.json").then((response) => response.json())
    .then((data) => {
    timestamp=Date.parse(data["time"]);
    let dt=(Date.now()-timestamp)/1000.0;
    dt=Math.round(dt * 10) / 10
    textContent="最近一次状态更新时间:   "+dt + " 秒之前";
    if (dt>45){
        textContent+="---警告：长时间未更新，检查仪器状态！";
        document.getElementById("timestamp").style.color="red";
    }
    document.getElementById("timestamp").textContent=textContent;
    //document.getElementById("timestamp").textContent="Updated:   "+timestamp;
    });   


},1000);


var t=setInterval(function(){
    fetch("/data/last_data_time.json").then((response)=>response.json()).then((data)=>{
        last_data_time=Date.parse(data['time']);
        let dt=(Date.now()-timestamp)/1000.0;
        dt=Math.round(dt * 10) / 10
        document.getElementById("data_ts").textContent="最近一次数据到达时间： "+ dt + " 秒之前";
    })
},1000);

setInterval(function(){
    document.getElementById("div_current_time").innerHTML=new Date();
}, 1000);