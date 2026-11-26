#!/usr/bin/env bash

sudo apt-get remove -y 'php.*'
sudo apt-get remove -y azure-cli google-cloud-sdk hhvm microsoft-edge-stable google-chrome-stable firefox powershell mono-devel
sudo apt-get autoremove -y
sudo apt-get clean

rm -rf /usr/share/dotnet/
