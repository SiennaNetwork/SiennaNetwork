if ! npx tsc -p . ; then
    exit
fi

node --trace-warnings ./dist/change_config.js $1
