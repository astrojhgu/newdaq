#!/usr/bin/env bash

function dev2id() {
    dev_name=$1 && sudo smartctl -a /dev/${dev_name} | grep -i wwn | awk -F ':' '{print $2}' | sed -e 's/\ //g'
}

function id2dev() {
    basename $(readlink /dev/disk/by-id/wwn-0x${1})
}

function dev2slot() {
    dev=$1
    wwn=$(dev2id ${dev}) && sudo storcli /c0/sall show all | grep -i ${wwn} -B 6 | head -1 | awk '{print $2}'
}

function slot2id() {
    slot=$1
    sudo storcli /c0 show all | grep Drive.*${slot}.*Device -A 6 | tail -1 | awk -F = '{print $2}' | sed 's/\ //g' | tr '[:upper:]' '[:lower:]'
}

function enumerate_slot() {
    sudo storcli /c0 show all | grep Drive.*State | awk '{print $2}'
}

function enumerate_dev() {
    for s in $(enumerate_slot); do
        id2dev $(slot2id $s)
    done
}

function format_dev() {
    echo formatting $1
    dev=$1

    s=$(dev2slot ${dev})
    sudo storcli ${s} stop locate

    sudo sgdisk -o /dev/${dev}
    sudo sgdisk -N 1 /dev/${dev}
    yes | sudo mkfs.ext4 /dev/${dev}1
    sudo mkdir -p /mnt/${dev}
    sudo mount /dev/${dev}1 /mnt/${dev}
    sudo chmod a+rwx /mnt/${dev}

    echo done
}

function eject_dev() {
    dev=$1
    s=$(dev2slot ${dev})

    echo ejecting $dev @ $s
    sudo umount /dev/${dev}1 && sudo storcli ${s} start locate && echo done
}

function format_all() {
    for d in $(enumerate_dev); do
        echo checking ${d} ...
        if [ -e /dev/${d} ]; then
            echo device $d found
            if df | grep /dev/${d} >/dev/null; then
                echo /dev/${d}1 mounted, no need to format
            elif [ -e /dev/${d}1 ]; then
                echo partation /dev/${d}1 found, no need to format
            else
                format_dev $d
            fi
        fi
    done
}

function eject_all() {
    echo ejecting all finished disks
    for d in $(enumerate_dev); do
        echo checking $d ...
        if df | grep /dev/${d}1 >/dev/null; then
            mpoint=$(df | grep /dev/${d}1 | awk '{print $6}')
            echo found $d @ $mpoint
            if [ -e ${mpoint}/done ]; then
                eject_dev ${d}
            else
                echo not yet finished
            fi
        else
            echo $d not mounted
        fi
    done
}

function inspect_disks() {
    devs=$(enumerate_dev)
    last_dev=$(echo $devs | awk '{print $NF}')
    echo "{" >/dev/shm/disk1.json
    for dev in $devs; do
        echo \"${dev}\": >>/dev/shm/disk1.json
        slot=$(dev2slot ${dev})
        if df | grep $dev >/dev/null; then
            occ=$(df | grep ${dev} | awk '{printf("%s /  %s   %s", $3,$2,$5)}')
            if [ -e /mnt/${dev}/time*txt ]; then
                state=Writing
            else
                state=Spare
            fi
        else
            state=Ejected
            occ=""
        fi
        echo "{" >>/dev/shm/disk1.json
        echo \"state\": \"$state\" , >>/dev/shm/disk1.json
        echo \"occ\": \"$occ\" , >>/dev/shm/disk1.json
        echo \"slot\": \"$slot\" >>/dev/shm/disk1.json
        echo $dev $state $occ
        echo "}" >>/dev/shm/disk1.json
        if [ x$last_dev == x$dev ]; then
            :
        else
            echo , >>/dev/shm/disk1.json
        fi
    done
    echo "}" >>/dev/shm/disk1.json
    mv -f /dev/shm/disk1.json /dev/shm/disk.json
}

while :; do
    date
    eject_all
    sleep 5
    format_all
    inspect_disks
    sleep 30
done
