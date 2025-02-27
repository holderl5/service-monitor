FROM rust:latest

ENV DEBIAN_FRONTEND noninteractive
ENV TZ=Asia/Kolkata DEBIAN_FRONTEND=noninteractive
RUN apt-get update && apt install tzdata -y && apt install net-tools vim man file jq less -y
RUN apt-get update -y && apt-get upgrade -y
RUN  apt-get install -y vim perl wget tar man sudo adduser netstat-nat net-tools curl w3m nodejs npm
RUN useradd -m  -s /bin/bash developer
RUN usermod -aG sudo developer && echo "developer ALL=(ALL) NOPASSWD: ALL" > /etc/sudoers.d/developer
RUN chmod 044 /etc/sudoers.d/developer
RUN mkdir -p /home/developer/app
USER developer:developer
WORKDIR /home/developer/app
RUN yes | npx wrangler help
RUN cargo install cargo-generate
CMD ["/bin/bash"]
