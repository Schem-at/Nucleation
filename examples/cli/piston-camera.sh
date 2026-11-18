#!/bin/sh
# Standard installed CLI only. Usage: NUCLEATION_PACK=/path/pack.zip ./piston-camera.sh /path/output
set -eu
out=${1:-./piston-camera-output}
mkdir -p "$out"
nucleation scene --name 'Piston and lever' --data-version 4671 \
  --set '0,0,0=minecraft:stone' \
  --set '0,0,1=minecraft:stone' \
  --set '0,1,0=minecraft:sticky_piston[facing=east,extended=false]' \
  --set '0,1,1=minecraft:lever[face=floor,facing=north,powered=false]' \
  --set '1,1,0=minecraft:gold_block' \
  -o "$out/piston.litematic"

nucleation render "$out/piston.litematic" -o "$out/piston.mp4" \
  --ticks 80 --use 10:0,1,1 --use 50:0,1,1 \
  --camera 0:45,30,1 --camera 80:225,40,1.2 \
  --width 1280 --height 720 --fps 60 --report "$out/result.json"
