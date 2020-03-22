#!/bin/bash

for f in `cat src/ext.rs | grep "feature=" | cut -d\" -f2`; do
    echo $f "= []"
done