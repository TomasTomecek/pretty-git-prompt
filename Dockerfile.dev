FROM tomastomecek/rust:clippy

LABEL maintainer="Tomas Tomecek <tomas@tomecek.net>"

# RUN dnf install -y curl tar gcc openssl-devel cmake make file libcurl-devel zsh && \
#     dnf clean all

USER root
# make               -- this project is using makefile
# zsh                -- 'duh
# cmake              -- git crate is compiled with cmake
# zlib-devel         -- git crate uses zlib library
# git python3-pytest -- integration tests
# python3-pexpect    -- demo
RUN dnf install -y make zsh cmake zlib-devel git python3-pytest python3-pexpect
USER rust

ARG RUST_BACKTRACE="1"
ENV PYTHONDONTWRITEBYTECODE=YES

RUN mkdir -p $HOME/.local/bin/ && \
    ln -s /src/target/debug/pretty-git-prompt $HOME/.local/bin/
COPY files/.zshrc /home/rust/.zshrc
COPY files/.bashrc /home/rust/.bashrc

ENV LANG=en_US.utf8 \
    LC_ALL=en_US.UTF-8 \
    PATH="$HOME/.local/bin/:${PATH}"

CMD ["/bin/zsh"]

COPY . /src
