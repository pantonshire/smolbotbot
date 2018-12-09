#!/bin/bash
/usr/local/bin/twurl "/1.1/statuses/show.json?id=${1}&include_entities=true&include_ext_alt_text=true"
