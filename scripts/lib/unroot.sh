#!/bin/sh
echo "looking around---------------------------"
cd ~
whoami
pwd
ls -alh *
echo "creating landing pad---------------------"
useradd -u 1000 -m not-root || true
mv ~/node_key.json /home/not-root/ || true
chown -R not-root /home/not-root
echo "pulling parachute------------------------"
su not-root -c $@
