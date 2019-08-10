#!/bin/bash

set -x

HOSTAP_SRC_PATH="./hostap"
HOSTAPD_PATH="hostapd"

do_action_prep() {
    git clone --depth 1 -b hostap_2_7 \
        git://w1.fi/hostap.git \
        ${HOSTAP_SRC_PATH} || echo "hostap repo already exists"
}

do_action_build() {
    cd ${HOSTAP_SRC_PATH}/hostapd
    if [ ! -f "${HOSTAPD_PATH}" ]; then
        cp ../tests/hwsim/example-hostapd.config .config
        make clean
        make -j3
    fi
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
