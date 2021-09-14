if ! npx tsc -p . ; then
    exit
fi

Commit=`git rev-parse --short HEAD`

node_modules/.bin/esmo --enable-source-maps ./deploy.ts $Commit $1
