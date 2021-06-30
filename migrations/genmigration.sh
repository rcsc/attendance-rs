#!/bin/bash

lowered_message=$(echo $1 | awk '{print tolower($0)}')
skewered_message=$(echo $lowered_message | sed "s/\s/-/g")
sqlfile=$(date +"%Y%m%d-%H%M%S")-$skewered_message.sql

touch $sqlfile
echo "Generated $sqlfile"
