if ! npx tsc -p . ; then
    exit
fi

node_modules/.bin/esmo --enable-source-maps ./change_config.ts $1
