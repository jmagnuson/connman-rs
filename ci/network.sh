#!/bin/bash

set -x

#  _____      _____        _____
# |     |    |     |      |     |
# | ETH |<-->| AP  |-))((-| STA |
# |_____|    |_____|      |_____|
#
ETHIF=ens4
STAIF=wlan0
APIF=wlan1
APIP="192.168.1.1/8"
STAIP="192.168.1.2/24"

do_action_prep() {
    sudo apt install \
        linux-modules-extra-`uname -r` \
        rfkill \
        iptables

    sudo modprobe mac80211_hwsim radios=2 fake_hw_scan=1

    sudo rfkill unblock all

    sudo iw reg set US

    # AP (used by hostapd)
    #sudo iw dev wlan1 set type __ap
    sudo ip link set dev ${APIF} up
    sudo ip address add dev ${APIF} ${APIP}

    # STA
    sudo ip link set dev ${STAIF} up
    sudo ip address add dev ${STAIF} ${STAIP}
}

action="$1"
shift
args="$@"
case $action in
    prep)
        do_action_prep
        ;;
    *)
        >&2 echo Unknown action $action
        exit 1
        ;;
esac
