#!/bin/bash

set -x

WPA_SUPPLICANT_SRC_PATH="./supplicant"
WPA_SUPPLICANT_PATH="wpa_supplicant"

do_action_prep() {
    sudo apt install \
        binutils-dev \
        libiberty-dev \
        libnl-3-dev \
        libnl-genl-3-dev \
        libnl-route-3-dev \
        libssl-dev

    git clone --depth 1 -b hostap_2_7 \
        git://w1.fi/hostap.git \
        ${WPA_SUPPLICANT_SRC_PATH} || echo "supplicant repo already exists"
}

do_action_build() {
    cd ${WPA_SUPPLICANT_SRC_PATH}/wpa_supplicant
    if [ ! -f "${WPA_SUPPLICANT_PATH}" ]; then
        cp ../tests/hwsim/example-wpa_supplicant.config .config
        make -j3
    fi
    sudo make install

    # Needed if we don't install from apt
    sudo cp dbus/fi.w1.wpa_supplicant1.service /usr/share/dbus-1/system-services/fi.w1.wpa_supplicant1.service
    sudo cp dbus/dbus-wpa_supplicant.conf /usr/share/dbus-1/system.d/dbus-wpa_supplicant.conf

    cd ../..
}

do_action_run() {
    sudo /usr/local/sbin/wpa_supplicant -B -dd -Dnl80211 -onl80211 -u -iwlan0 -c ci/wpa_supplicant.conf
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
