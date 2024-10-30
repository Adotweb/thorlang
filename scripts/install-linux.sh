#!/bin/bash

#create and enter some temporary directory for the gzipped archive
mkdir ~/.temp-thorlang
cd ~/.temp-thorlang

echo "installing linux binary"

#install archive
curl -s "https://api.github.com/repos/Adotweb/thorlang/releases/latest" \
| grep "browser_download_url.*tar" \
| cut -d '"' -f 4 \
| wget -qi-

#unpack archive
tar -xzvf thorlang-linux.tar.gz

#move executable to binaries
mv thorlang-linux /usr/bin/thorlang


#cleanup
cd ..
rm -r ~/.temp-thorlang

echo "done installing"
