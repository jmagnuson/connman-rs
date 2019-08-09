#!/bin/bash

set -x

do_action_prep() {
    sudo apt install \
        linux-modules-extra-`uname -r` \
        rfkill

    sudo modprobe mac80211_hwsim radios=2 fake_hw_scan=1

    sudo rfkill unblock all

    sudo iw reg set US

    # STA
    sudo ifconfig wlan0 up 192.168.1.2

    # AP (used by hostapd)
    sudo ifconfig wlan1 up 192.168.1.1
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
