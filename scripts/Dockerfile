FROM node:lts-alpine
RUN ls -al
RUN apk update
RUN apk add docker
RUN mkdir -p /sienna
WORKDIR /sienna
ADD package.json yarn.lock ./
RUN yarn
