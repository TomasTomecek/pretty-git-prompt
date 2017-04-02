# .bashrc

# Source global definitions
if [ -f /etc/bashrc ]; then
  . /etc/bashrc
fi

pretty_prompt() { PS1="$(pretty-git-prompt)\n\$ "; }
# workaround: https://superuser.com/a/623305/160542
#             http://stackoverflow.com/a/13997892/909579
export PROMPT_COMMAND="pretty_prompt ; $PROMPT_COMMAND"
