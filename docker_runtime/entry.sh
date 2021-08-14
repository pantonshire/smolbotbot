#!/bin/sh

true > sbb.crontab

if [ "$SBB_TIMELINE" = 'true' ]; then
    cat timeline.crontab >> sbb.crontab
fi

if [ "$SBB_DAILY" = 'true' ]; then
    cat daily.crontab >> sbb.crontab
fi

if [ "$SBB_SAVEIDS" = 'true' ]; then
    cat saveids.crontab >> sbb.crontab
fi

echo 'Installing crontab'
crontab sbb.crontab

if [ -n "$SBB_BOOTSTRAP_IDS" ]; then
    echo 'Bootstrapping from provided tweet ids'
    echo "$SBB_BOOTSTRAP_IDS" | sbb fetch | sbb image --connect-timeout 30 --request-timeout 300 --thumb-size 192 -dt /var/lib/smolbotbot/images ids
fi

if [ -n "$SBB_BOOTSTRAP_URL" ]; then
    echo 'Bootstrapping from provided url'
    curl -f "$SBB_BOOTSTRAP_URL" | sbb fetch | sbb image --connect-timeout 30 --request-timeout 300 --thumb-size 192 -dt /var/lib/smolbotbot/images ids
fi

echo 'Smolbotbot is now ready'
echo 'Starting cron'
exec /usr/sbin/crond -f
