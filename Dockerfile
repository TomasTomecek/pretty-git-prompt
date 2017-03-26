FROM fedora:25

LABEL maintainer="Tomas Tomecek <tomas@tomecek.net>"

RUN dnf install -y curl tar gcc openssl-devel cmake make file libcurl-devel zsh && \
    dnf clean all

# beta, nightly, 1.15.1
# channel, channel + date, or an explicit version
ARG RUST_SPEC=nightly
ARG WITH_TEST="yes"
ARG USER_ID="1000"
ARG RUST_BACKTRACE="1"

ENV HOME=/home/pretty
RUN useradd -o -u ${USER_ID} -m pretty

# https://static.rust-lang.org/dist/2017-03-16/rust-nightly-x86_64-unknown-linux-gnu.tar.gz
RUN cd $HOME && curl -s https://static.rust-lang.org/rustup.sh | sh -s -- --spec=$RUST_SPEC --verbose --disable-sudo

# leave this here in case the script above breaks
# ENV RUST_ARCHIVE=rust-$RUST_CHANNEL-x86_64-unknown-linux-gnu.tar.gz
# ENV RUST_DOWNLOAD_URL=https://static.rust-lang.org/dist/$RUST_ARCHIVE
# RUN set -x && mkdir /rust && cd /rust && \
#     curl -fsOSL $RUST_DOWNLOAD_URL && \
#     curl -s $RUST_DOWNLOAD_URL.sha256 | sha256sum -c - && \
#     tar -C /rust -xzf $RUST_ARCHIVE --strip-components=1 && \
#     ./install.sh && \
#     rm -rf /rust

RUN if [ $WITH_TEST == "yes" ] ; then \
    cargo install clippy || : && \
    dnf install -y git python3-pytest ; \
    fi

USER ${USER_ID}

RUN mkdir -p $HOME/.local/bin/ && \
    ln -s /app/target/debug/pretty-git-prompt $HOME/.local/bin/
COPY files/.zshrc /home/pretty/.zshrc

ENV LANG=en_US.utf8 \
    LC_ALL=en_US.UTF-8 \
    PATH="$HOME/.local/bin/:${PATH}"

CMD ["/bin/zsh"]

WORKDIR /app

COPY . /app
