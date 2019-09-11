#!/bin/bash

set -x

HOSTAP_SRC_PATH="./hostap"

do_action_prep() {
    sudo apt install hostapd
}

do_action_build() {
    cd ${HOSTAP_SRC_PATH}/hostapd
    cp defconfig .config
    make clean
    make -j3
    sudo make install
    cd ../..
}

do_action_run() {
    sudo /usr/local/bin/hostapd -B -ddt ci/hostapd.conf
}

action="$1"
shift
args="$@"
case $action in
    prep)
        do_action_prep
        ;;
    build)
        do_action_build
        ;;
    run)
        do_action_run
        ;;
    *)
        >&2 echo Unknown action $action
        exit 1
        ;;
esac
