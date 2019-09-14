#!/bin/bash

set -x

HOSTAP_SRC_PATH="./hostap"

do_action_prep() {
    sudo apt install \
        libnl-3-dev \
        libnl-genl-3-dev

    sudo apt install wpasupplicant

    git clone --depth 1 -b hostap_2_7 \
        git://w1.fi/hostap.git \
        ${HOSTAP_SRC_PATH} || echo "hostap/supplicant repo already exists"
}

do_action_build() {
    cd ${HOSTAP_SRC_PATH}/wpa_supplicant
    cp defconfig .config
    # Enable dbus
    cat >> .config << "EOF"
CONFIG_CTRL_IFACE_DBUS=y
CONFIG_CTRL_IFACE_DBUS_NEW=y
CONFIG_CTRL_IFACE_DBUS_INTRO=y
EOF

    make -j3
    sudo make install
    cd ../..
}

do_action_run() {
    sudo /usr/local/sbin/wpa_supplicant -B -dd -Dnl80211 -u -iwlan0 -c ci/wpa_supplicant.conf
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
