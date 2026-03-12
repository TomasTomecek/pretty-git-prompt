FROM fedora:43

LABEL maintainer="Tomas Tomecek <tomas@tomecek.net>"

# make               -- this project is using makefile
# zsh                -- 'duh
# cmake              -- git crate is compiled with cmake
# zlib-devel         -- git crate uses zlib library
# git python3-pytest -- integration tests
# python3-pexpect    -- demo
RUN dnf install -y make zsh cmake zlib-devel git python3-pytest python3-pexpect cargo clippy

ARG RUST_BACKTRACE="1"
ENV PYTHONDONTWRITEBYTECODE=YES

RUN mkdir -p /root/.local/bin/ && \
    ln -s /src/target/debug/pretty-git-prompt /root/.local/bin/
COPY files/.zshrc /root/.zshrc
COPY files/.bashrc /root/.bashrc

ENV LANG=en_US.utf8 \
    LC_ALL=en_US.UTF-8 \
    PATH="/root/.local/bin/:${PATH}"

CMD ["/bin/zsh"]

COPY . /src
