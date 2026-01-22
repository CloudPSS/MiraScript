FROM docker.cloudpss.net/cloudpss/buildpack-lts AS builder

RUN mkdir -p /core

RUN apt update\
    && apt install -y software-properties-common\
    && add-apt-repository ppa:deadsnakes/ppa \
    && apt update \
    && apt install -y python3.7 

RUN pip3 install Virtualenv \
    && virtualenv -p python3.7 /opt/venv 

RUN /opt/venv/bin/pip install maturin==1.7.5

WORKDIR /core



