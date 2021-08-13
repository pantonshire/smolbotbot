#!/bin/sh

sbb timeline -n 20 '@smolrobots' | sbb image --connect-timeout 30 --request-timeout 300 --thumb-size 192 -dt /var/lib/smolbotbot/images ids
sbb post daily --cleanup

# exec /usr/sbin/crond -f
