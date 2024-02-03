#!/bin/sh

cargo build
# default installation in /opt/augre/
sudo mkdir /opt/augre/
sudo cp target/debug/augre /opt/augre/
sudo cp target/debug/config.toml /opt/augre/
# adding "augre" accesible anywhere
echo 'alias augre="/opt/augre/augre"' >> ~/.bashrc
source ~/.bashrc
