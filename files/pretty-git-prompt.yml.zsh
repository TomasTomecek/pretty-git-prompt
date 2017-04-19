# version of configuration file
# right now it needs to be set to '1'
version: '1'
# configuration of various values (required), type dict
# if you omit a value, it won't be displayed
values:
    # count of untracked files
    new:
        # formatting (required), both (pre_format, post_format) are required
        # you can include coloring in pre_format and reset colors in post_format
        # you can also include arbitrary string
        # for more information about setting colors for bash and zsh:
        # https://wiki.archlinux.org/index.php/zsh#Colors
        # http://zsh.sourceforge.net/Doc/Release/Prompt-Expansion.html#Visual-effects
        # https://www.ibm.com/developerworks/linux/library/l-tip-prompt/
        # this is how the value is formatted in the end:
        #   [pre_format][value][post_format]
        # example:
        #   ✚2
        pre_format: '%{%F{014}%}✚'
        post_format: '%{%f%}'
    # the number of tracked files which were changed in working tree
    changed:
        pre_format: '%{%B%F{red}%}Δ'
        post_format: '%{%b%f%}'
    # the number of files added to index
    staged:
        pre_format: '%{%F{green}%}▶'
        post_format: '%{%f%}'
    # during merge, rebase, or others, the numbers files which conflict
    conflicts:
        pre_format: '%{%F{yellow}%}✖'
        post_format: '%{%f%}'
    # the number of files present locally which are missing in remote repo
    difference_ahead:
        pre_format: '%{%F{white}%}↑'
        post_format: '%{%f%}'
    # the number of commits present in remote repo which are missing locally
    difference_behind:
        pre_format: '%{%F{white}%}↓'
        post_format: '%{%f%}'

# monitor status against different remotes (optional), type dict
# track history divergence
monitor_remotes:
      # there are some special values which are substituted:
      #  * <REMOTE> will be replaced with name of a remote
      #  * <LOCAL_BRANCH> will be replaced with current branch name
      #  * <REMOTE_BRANCH> will be replaced with name of remote branch
    - pre_format: '%{%F{blue}%}<LOCAL_BRANCH>'
      post_format: '%{%f%}'
      # remote branch name (optional), type string
      # example: 'upstream/master'
      # if omitted look for remotely tracked branch usualy set up with:
      #   git branch --set-upstream-to
      # remote_branch: ''
      # display the remote even if there is no difference with current branch (required), type bool
      display_if_uptodate: true
    - remote_branch: 'upstream/master'
      display_if_uptodate: false
      pre_format: '%{%F{green}%}<REMOTE>'
      post_format: '%{%f%}'
