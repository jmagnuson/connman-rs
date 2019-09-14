#!/bin/bash

set -x

CONNMAN_SRC_PATH="./connman"
CONNMAND_PATH="src/connmand"

do_action_prep() {
    sudo apt install \
        dbus \
        libdbus-1-dev \
        gnutls-dev \
        gnutls-bin \
        xtables-addons-common \
        xtables-addons-source \

    # TODO: Install deps to be able to run stock `bootstrap-config`
    #sudo apt install openconnect
    #sudo apt install openvpn
    #sudo apt install nftables
    #sudo apt install libnftnl-dev
    #sudo apt install vpnc

    git clone --depth 1 -b 1.36 \
        https://git.kernel.org/pub/scm/network/connman/connman.git \
        ${CONNMAN_SRC_PATH} || echo "connman repo already exists"
}

do_action_build() {
    cd ${CONNMAN_SRC_PATH}

    # only build if daemon doesn't already exist (from cache)
    if [ ! -f "${CONNMAND_PATH}" ]; then
        ./bootstrap
        ./configure
        make
    fi
    sudo make install
    cd ..

    # This may be done automatically if using `bootstrap-config`
    sudo cp connman/src/connman-dbus.conf /etc/dbus-1/system.d/
    sudo systemctl reload dbus
}

do_action_run() {
    sudo connmand -d --nodnsproxy -i wlan0

    # Make sure wifi is enabled
    sudo connmanctl enable wifi
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
