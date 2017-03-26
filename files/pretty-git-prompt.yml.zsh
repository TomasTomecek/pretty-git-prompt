# configuration of various values (required), type dict
# if you omit a value, it won't be displayed
values:
    # count of untracked files
    new:
        # prefix label (required), type string
        label: '✚'
        # formatting specification of the label and value
        # https://wiki.archlinux.org/index.php/zsh#Colors
        # http://zsh.sourceforge.net/Doc/Release/Prompt-Expansion.html#Visual-effects
        # TODO: bash
        # TODO: fish?
        pre_format: '%{%F{014}%}'
        post_format: '%{%f%}'
    changed:
        label: 'Δ'
        pre_format: '%{%B%F{red}%}'
        post_format: '%{%b%f%}'
    staged:
        label: '▶'
        pre_format: '%{%F{green}%}'
        post_format: '%{%f%}'
    conflicts:
        label: '✖'
        pre_format: '%{%F{yellow}%}'
        post_format: '%{%f%}'
    difference_ahead:
        label: '↑'
        pre_format: '%{%F{white}%}'
        post_format: '%{%f%}'
    difference_behind:
        label: '↓'
        pre_format: '%{%F{white}%}'
        post_format: '%{%f%}'

# monitor status against different remotes (optional), type dict
# track history divergence
monitor_remotes:
    origin:
        # display the remote even if there is no difference with current branch (required), type bool
        display_if_uptodate: true
        # this is displayed as: '[pre_format][value][post_format]'
        # include coloring in pre_format and reset colors in post_format
        # you can also include arbitrary string and substitute special values:
        #  * <REMOTE> will be replaced with name of a remote
        #  * <BRANCH> will be replaced with current branch name
        pre_format: '%{%F{blue}%}<BRANCH>'
        post_format: '%{%f%}'
    # remote name (optional), type dict
    upstream:
        # remote branch name (optional), type string
        # if omitted look for remotely tracked one
        # git branch --set-upstream-to
        branch: master
        display_if_uptodate: false
        pre_format: '%{%F{green}%}<REMOTE>'
        post_format: '%{%f%}'

