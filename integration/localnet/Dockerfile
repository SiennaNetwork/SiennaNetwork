FROM enigmampc/secret-network-sw-dev
WORKDIR /tmp
ADD https://github.com/wolfcw/libfaketime/archive/refs/tags/v0.9.9.tar.gz .
RUN tar -xvf v0.9.9.tar.gz && cd libfaketime-0.9.9 && make && make install
WORKDIR /root
