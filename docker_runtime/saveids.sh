#!/bin/sh
if [ -e /var/lib/smolbotbot/bootstrap/ids ]; then
    cp /var/lib/smolbotbot/bootstrap/ids /var/lib/smolbotbot/bootstrap/ids.bak
fi
sbb export > /var/lib/smolbotbot/bootstrap/ids
