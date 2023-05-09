#!/usr/bin/bash

#check if target exists, if not, build it.
if [ ! -d "./target/release" ]
then
    cargo build --release
fi
#start backend 
nohup ./target/release/todo_api &

WEB_HOST=localhost
WEB_PORT=8080
WEB_SRC=dist

#get web server port and host from .env file
while read line; do 
    KEY=${line%=*}
    VALUE=${line##*=}
    if [ $KEY = "WEB_HOST" ]; then 
        WEB_HOST=$VALUE
    fi
    if [ $KEY = "WEB_PORT" ]; then 
        WEB_PORT=$VALUE
    fi
    if [ $KEY = "WEB_SRC" ]; then 
        WEB_SRC=$VALUE
    fi
done< <(cat .env)

echo $WEB_HOST
echo $WEB_PORT
echo $WEB_SRC


#start web server for ui
nohup ./serve_app -p $WEB_PORT -h $WEB_HOST -s $WEB_SRC &






