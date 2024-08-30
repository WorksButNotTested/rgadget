################################################################################
# DEVCONTAINER                                                                 #
################################################################################
FROM ubuntu:20.04 AS devcontainer
ARG http_proxy
ARG https_proxy

ENV http_proxy=$http_proxy
ENV https_proxy=$https_proxy

ENV DEBIAN_FRONTEND="noninteractive"
RUN apt-get update && \
    apt-get install -y \
    curl \
    git \
    git-lfs

# Install latest version of Visual Studio Code server
ENV COMMIT_ID=f1e16e1e6214d7c44d078b1f0607b2388f29d729
WORKDIR /root/.vscode-server/bin
RUN curl -sL https://update.code.visualstudio.com/commit:${COMMIT_ID}/server-linux-x64/stable -o stable
RUN tar xf stable
RUN mv vscode-server-linux-x64 ${COMMIT_ID}

################################################################################
# RGADGET                                                                      #
################################################################################
FROM devcontainer AS rgadget
RUN apt-get update && \
    apt-get install -y \
    build-essential \
    gdb

# Install rust v1.79
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain 1.79
ENV PATH="${PATH}:/root/.cargo/bin"

# Install samply
RUN cargo install samply

# Install Node.js and NPM
RUN curl -sL https://deb.nodesource.com/setup_18.x | bash -
RUN apt-get install -y nodejs
RUN npm install -g yarn

# Install Firefox Profiler
WORKDIR /root/
RUN git clone https://github.com/firefox-devtools/profiler.git
WORKDIR /root/profiler
RUN yarn install
ENV FX_PROFILER_HOST=127.0.0.1
