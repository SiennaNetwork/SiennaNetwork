FROM node:lts-slim

RUN apt update -y && apt upgrade -y
RUN apt install -y docker netcat git docker

ADD https://github.com/enigmampc/SecretNetwork/releases/download/v1.0.4/secretcli-linux-amd64 /usr/local/bin/secretcli
RUN chmod +x /usr/local/bin/secretcli

RUN mkdir -p /sienna
WORKDIR /sienna
ADD package.json yarn.lock ./
RUN yarn
