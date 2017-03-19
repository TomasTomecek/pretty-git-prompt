#!/bin/bash
#
# This is meant to run in a container.

mkdir -p ~/.local/bin/
ln -s /app/target/debug/pretty-git-prompt ~/.local/bin/
export PATH="/home/pretty/.local/bin/:${PATH}"

cat >>~/.zshrc <<EOF
export LC_ALL=en_US.UTF-8
# Load colors.
autoload -U colors
colors
# Allow for functions in the prompt.
setopt PROMPT_SUBST
RPROMPT='\$(pretty-git-prompt -m zsh)'

EOF

exec python3 tests/integration/utils.py
