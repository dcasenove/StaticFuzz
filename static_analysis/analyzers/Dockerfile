FROM ubuntu:20.04

ENV DEBIAN_FRONTEND=noninteractive

RUN DEBIAN_FRONTEND=noninteractive apt-get update && \
    apt-get -y upgrade && \
    apt-get install -y git gnupg zlib1g-dev python3 golang-go python3-pip python3-dev python-is-python3 wget curl build-essential cmake zip vim nodejs npm openjdk-11-jdk && \
    apt-get clean

COPY . /usr/src/analyzers
WORKDIR /usr/src/analyzers
RUN chmod +x ./get_analyzers.sh
RUN ./get_analyzers.sh

CMD ["tail", "-f", "/dev/null"]
