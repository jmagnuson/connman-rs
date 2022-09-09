#!/bin/bash

set -ex

# from: https://github.com/openweave/happy/blob/cbf93f74eb64a083fbb213d2140e7df9a2d7bc64/.github/workflows/main.yml

AZURE_KERNEL_VER="5.15.0-1019-azure"

# i don't think we want to update
#sudo apt-get upgrade
#sudo apt-get update

echo show linux-image-extra
sudo apt-get install wireless-crda
#curl http://archive.ubuntu.com/ubuntu/pool/main/l/linux-azure/linux-modules-extra-5.4.0-1020-azure_5.4.0-1020.20_amd64.deb --output linux-modules-extra-5.4.0-1020-azure_5.4.0-1020.20_amd64.deb

#sudo apt-get install linux-generic

echo try to instaall linux-modules-extra-5.4.0-1020-azure_5.4.0-1020.20_amd64
curl http://archive.ubuntu.com/ubuntu/pool/main/l/linux-azure/linux-modules-extra-5.15.0-1019-azure_5.15.0-1019.24_amd64.deb \
  --output linux-modules-extra-5.15.0-1019-azure_5.15.0-1019.24_amd64.deb
sudo dpkg -i linux-modules-extra-5.15.0-1019-azure_5.15.0-1019.24_amd64.deb

sudo apt-get install linux-image-extra-virtual-hwe-20.04

cd /lib/modules/
ls -lh
echo look for mac80211_hwsim.ko
find . -name mac80211_hwsim.ko

# TODO: use the result of `find`, since 5.15.0-XX could change
# originally threw: modprobe: FATAL: Module mac80211_hwsim not found in directory /lib/modules/5.15.0-1019-azure
HWSIM_MODULE_PATH="kernel/drivers/net/wireless"
HWSIM_MODULE_NAME="mac80211_hwsim.ko"
sudo mkdir -p /lib/modules/$AZURE_KERNEL_VER/${HWSIM_MODULE_PATH}
sudo ln -s /lib/modules/5.15.0-46-generic/${HWSIM_MODULE_PATH}/${HWSIM_MODULE_NAME} /lib/modules/${AZURE_KERNEL_VER}/${HWSIM_MODULE_PATH}/${HWSIM_MODULE_NAME}
find . -name mac80211_hwsim.ko
#sudo ln -s /lib/modules/5.15.0-46-generic/kernel/drivers/net/wireless/mac80211_hwsim.ko /lib/modules/5.15.0-1019-azure/kernel/drivers/net/wireless/

sudo depmod -a
sudo modprobe mac80211_hwsim || echo 'modprobe failed, try insmod' # && find . -name mac80211_hwsim.ko -exec sudo insmod -f {} \;
sudo insmod -f /lib/modules/5.15.0-46-generic/kernel/drivers/net/wireless/mac80211_hwsim.ko

echo "search again"
cd 5.15.0-1019-azure
find .
echo check ip netns
sudo ip netns add blue
sudo ip netns list
sudo ip netns del blue
echo check 80211.
sudo modprobe mac80211_hwsim
sudo lsmod | grep  mac80211_hwsim
