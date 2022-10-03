console.log("hello")

fetch("/data/mode.json").then((response) => response.json())
.then((data) => {
    if (data["mode"]==3){
        document.getElementById("status").textContent="运行中"
    }else{
        document.getElementById("status").textContent="停止"
    }
});

fetch("/data/temperature.json").then((response) => response.json())
.then((data) => {
    temperature=data["temperature"];
    console.log(temperature);
    temperature_html="<p>";
    for(i=0;i<5;++i){
        temperature_html+="板卡 "+i+" : ";
        temperature_html+=temperature[i*2]+" "+temperature[i*2+1];
        temperature_html+="</p>"
    }
    document.getElementById("div_health").innerHTML=temperature_html;
});

var timestamp;
fetch("/data/last_msg_time.json").then((response) => response.json())
.then((data) => {
    timestamp=Date.parse(data["time"]);
    //document.getElementById("timestamp").textContent="Updated:   "+timestamp;
});

var t=setInterval(function(){
    dt=(Date.now()-timestamp)/1000.0;
    textContent="上次更新时间:   "+dt+ " 秒之前";
    if (dt>15){
        textContent+="---警告：长时间未更新，检查仪器状态！";
        document.getElementById("timestamp").style.color="red";
    }
    document.getElementById("timestamp").textContent=textContent;


},1000);