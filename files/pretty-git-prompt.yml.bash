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
        pre_format: '\[\e[38;5;248m\]✚'
        post_format: '\[\e[0m\]'
    # the number of tracked files which were changed in working tree
    changed:
        pre_format: '\[\e[38;5;166m\]Δ'
        post_format: '\[\e[0m\]'
    # the number of files added to index
    staged:
        pre_format: '\[\e[38;5;2m\]▶'
        post_format: '\[\e[0m\]'
    # during merge, rebase, or others, the numbers files which conflict
    conflicts:
        pre_format: '\[\e[38;5;226m\]✖'
        post_format: '\[\e[0m\]'
    # the number of files present locally which are missing in remote repo
    difference_ahead:
        pre_format: '\[\e[38;5;7m\]↑'
        post_format: '\[\e[0m\]'
    # the number of commits present in remote repo which are missing locally
    difference_behind:
        pre_format: '\[\e[38;5;7m\]↓'
        post_format: '\[\e[0m\]'

# monitor status against different remotes (optional), type dict
# track history divergence
monitor_remotes:
      # there are some special values which are substituted:
      #  * <REMOTE> will be replaced with name of a remote
      #  * <LOCAL_BRANCH> will be replaced with current branch name
      #  * <REMOTE_BRANCH> will be replaced with name of remote branch
    - pre_format: '\[\e[38;5;4m\]<LOCAL_BRANCH>'
      post_format: '\[\e[0m\]'
      # remote branch name (optional), type string
      # example: 'upstream/master'
      # if omitted look for remotely tracked branch usualy set up with:
      #   git branch --set-upstream-to
      # remote_branch: ''
      # display the remote even if there is no difference with current branch (required), type bool
      display_if_uptodate: true
    - remote_branch: 'upstream/master'
      display_if_uptodate: false
      pre_format: '\[\e[38;5;2m\]<REMOTE>'
      post_format: '\[\e[0m\]'
