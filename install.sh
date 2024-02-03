#!/bin/sh

cargo build --release
# default installation in /opt/augre/
sudo mkdir /opt/augre/
sudo cp target/release/augre /opt/augre/
sudo cp target/release/config.toml /opt/augre/
# adding "augre" accesible anywhere
echo 'alias augre="/opt/augre/augre"' >> ~/.bashrc
source ~/.bashrc
