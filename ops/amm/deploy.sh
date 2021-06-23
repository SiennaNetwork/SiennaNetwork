if ! npx tsc -p . ; then
    exit
fi

Commit=`git rev-parse --short HEAD`

node --trace-warnings ./dist/deploy.js $Commit $1
