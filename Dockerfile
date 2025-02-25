FROM rust:latest

ENV DEBIAN_FRONTEND noninteractive
ENV TZ=Asia/Kolkata DEBIAN_FRONTEND=noninteractive
RUN apt-get update && apt install tzdata -y && apt install net-tools vim man file -y
RUN apt-get update -y && apt-get upgrade -y
RUN  apt-get install -y vim perl wget tar man sudo adduser netstat-nat net-tools curl w3m nodejs
RUN useradd -m  -s /bin/bash ubuntu
RUN usermod -aG sudo ubuntu && echo "ubuntu ALL=(ALL) NOPASSWD: ALL" > /etc/sudoers.d/ubuntu
RUN chmod 044 /etc/sudoers.d/ubuntu
RUN mkdir /home/ubuntu/app
USER ubuntu:ubuntu
WORKDIR /home/ubuntu/app
CMD ["/bin/bash"]
