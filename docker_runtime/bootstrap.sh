#!/bin/sh

if test -n "$SBB_BOOTSTRAP_IDS"; then
    echo "$SBB_BOOTSTRAP_IDS" \
        | sbb fetch \
        | sbb image \
            --connect-timeout 30 \
            --request-timeout 300 \
            --thumb-size 192 \
            -dt \
            /var/lib/smolbotbot/images \
            ids
fi

if test -n "$SBB_BOOTSTRAP_URL"; then
    wget -q -O - "$SBB_BOOTSTRAP_URL" \
        | sbb fetch \
        | sbb image \
            --connect-timeout 30 \
            --request-timeout 300 \
            --thumb-size 192 \
            -dt \
            /var/lib/smolbotbot/images \
            ids
fi
