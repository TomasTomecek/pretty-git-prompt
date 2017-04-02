# configuration of various values (required), type dict
# if you omit a value, it won't be displayed
values:
    # count of untracked files
    new:
        # https://wiki.archlinux.org/index.php/zsh#Colors
        # http://zsh.sourceforge.net/Doc/Release/Prompt-Expansion.html#Visual-effects
        # https://www.ibm.com/developerworks/linux/library/l-tip-prompt/
        pre_format: '%{%F{014}%}✚'
        post_format: '%{%f%}'
    changed:
        pre_format: '%{%B%F{red}%}Δ'
        post_format: '%{%b%f%}'
    staged:
        pre_format: '%{%F{green}%}▶'
        post_format: '%{%f%}'
    conflicts:
        pre_format: '%{%F{yellow}%}✖'
        post_format: '%{%f%}'
    difference_ahead:
        pre_format: '%{%F{white}%}↑'
        post_format: '%{%f%}'
    difference_behind:
        pre_format: '%{%F{white}%}↓'
        post_format: '%{%f%}'

# monitor status against different remotes (optional), type dict
# track history divergence
monitor_remotes:
      # formatting (required), both are required
      # this is displayed as: '[pre_format][value][post_format]'
      # include coloring in pre_format and reset colors in post_format
      # you can also include arbitrary string
      # there are some special values which are substituted:
      #  * <REMOTE> will be replaced with name of a remote
      #  * <LOCAL_BRANCH> will be replaced with current branch name
      #  * <REMOTE_BRANCH> will be replaced with name of remote branch
    - pre_format: '%{%F{blue}%}<LOCAL_BRANCH>'
      post_format: '%{%f%}'
      # remote branch name (optional), type string
      # example: 'upstream/mater'
      # if omitted look for remotely tracked branch usualy set up with:
      #   git branch --set-upstream-to
      # remote_branch: ''
      # display the remote even if there is no difference with current branch (required), type bool
      display_if_uptodate: true
    - remote_branch: 'upstream/master'
      display_if_uptodate: false
      pre_format: '%{%F{green}%}<REMOTE>'
      post_format: '%{%f%}'
