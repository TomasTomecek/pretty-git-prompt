# pretty-git-prompt

[![release](https://github.com/TomasTomecek/pretty-git-prompt/actions/workflows/release.yml/badge.svg)](https://github.com/TomasTomecek/pretty-git-prompt/actions/workflows/release.yml)
[![crates.io](https://img.shields.io/crates/v/pretty-git-prompt.svg)](https://crates.io/crates/pretty-git-prompt)
[![license](https://img.shields.io/crates/l/pretty-git-prompt.svg)](./LICENSE)

Your current git repository information inside a beautiful shell prompt.

![Preview of pretty-git-prompt](/data/example.png)

Features:

 * You are able to display values such as:
   * git repository state (resolving `merge` conflict, interactive `rebase`, ...)
   * Current branch name, or the commit hash when the `HEAD` is detached.
   * Name of a tag which points at the checked out commit.
   * Count of changed, newly-added, staged, conflicting files.
   * Number of items in stash.
   * Divergence (ahead/behind) against the tracked branch or an arbitrary
     remote branch.
 * Every value in output can be fully configured via a config file.
 * Sample configuration files feature colors, and `pretty-git-prompt
   list-colors` and `pretty-git-prompt preview` let you pick and check them
   without touching your shell config.
 * The tool supports `zsh` and `bash`.
 * pretty-git-prompt is written in Rust programming language and is delivered as a single, statically-linked binary.


## Table of contents

* [Development status](#development-status)
* [How can I try this out?](#how-can-i-try-this-out)
* [Installation](#installation)
  * [Obtaining `pretty-git-prompt` binary](#obtaining-pretty-git-prompt-binary)
    * [Fedora](#fedora)
    * [crates.io](#cratesio)
    * [GitHub release](#github-release)
    * [Compile it yourself](#compile-it-yourself)
  * [shell configuration](#shell-configuration)
    * [zsh](#zsh-1)
    * [bash](#bash-1)
  * [Skipping selected repositories](#skipping-selected-repositories)
* [Configuration](#configuration)
  * [Where the config file lives](#where-the-config-file-lives)
  * [Picking colors](#picking-colors)
* [Command line interface](#command-line-interface)
* [Solving problems](#solving-problems)
* [Contributing](#contributing)
* [Credits](#credits)


## Development status

The tool is ready to use. The latest release is
[0.3.0](https://github.com/TomasTomecek/pretty-git-prompt/releases/latest).


## How can I try this out?

Very easily! You don't need to install pretty-git-prompt if you just want to
see it in action. There is a make target which launches a container with the
whole environment set up (it needs `podman` and `make`).

It just takes some time to prepare the environment (create build environment,
compile the tool, run the demo).

Just clone this git repository

```
$ git clone https://github.com/TomasTomecek/pretty-git-prompt
```

and run...


### zsh

```
$ make zsh-demo
```

And this is what you should see:

![Preview using zsh.](/data/zsh-screenshot.png)

This is an interactive shell, so you can play with it.


### bash

In case you want to see the tool in bash shell:

```
$ make bash-demo
```

![Preview using bash.](/data/bash-screenshot.png)

This demo is one of the ways I verify that the tool works correctly.


## Installation

If you want to add pretty-git-prompt inside your shell, this section contains
information how to do that.


### Obtaining `pretty-git-prompt` binary


#### Fedora

pretty-git-prompt is packaged in Fedora:

```
$ sudo dnf install pretty-git-prompt
```


#### crates.io

Every release is published to [crates.io](https://crates.io/crates/pretty-git-prompt),
so if you have `cargo` available:

```
$ cargo install pretty-git-prompt
```

The binary lands in `~/.cargo/bin/`.


#### GitHub release

Every [GitHub release](https://github.com/TomasTomecek/pretty-git-prompt/releases/latest)
has binaries attached for `x86_64-unknown-linux-gnu`, `x86_64-apple-darwin` and
`aarch64-apple-darwin`, named `pretty-git-prompt-${VERSION}-${TARGET}`:

```
$ curl -LO https://github.com/TomasTomecek/pretty-git-prompt/releases/download/0.3.0/pretty-git-prompt-0.3.0-x86_64-unknown-linux-gnu
$ install -m 0755 pretty-git-prompt-0.3.0-x86_64-unknown-linux-gnu ~/.local/bin/pretty-git-prompt
```

On an Apple silicon Mac the asset to fetch is
`pretty-git-prompt-0.3.0-aarch64-apple-darwin`, on an Intel one
`pretty-git-prompt-0.3.0-x86_64-apple-darwin`.


#### Compile it yourself

```
$ make build
```

As stated inside demo section above, this takes some time.

If you have rust compiler and cargo available on your system, you can compile
the tool without using a container:

```
$ make exec-release-build
```

The binary is then available on this path:

```
$ ls -lha target/release/pretty-git-prompt
-rwxr-xr-x 2 user group 1.7M May  9 21:37 target/release/pretty-git-prompt
```


### shell configuration

Before digging into `.bashrc` and `.zshrc`, please make sure that binary
`pretty-git-prompt` is placed on your `$PATH`:

```
$ pretty-git-prompt
master│✚1Δ1
```

### zsh

This seems to be the minimal config required:

```shell
export LC_ALL=en_US.UTF-8
# Load colors.
autoload -U colors
colors
# Allow for functions in the prompt.
setopt PROMPT_SUBST
RPROMPT='$(pretty-git-prompt)'
```

Just put it inside your `~/.zshrc` and try it out.


### bash

You should paste this inside your `~/.bashrc`:

```
pretty_prompt() { PS1="$(pretty-git-prompt)\n\$ "; }
export PROMPT_COMMAND="pretty_prompt ; $PROMPT_COMMAND"
```

For more info about the presented solution, please read these [superuser.com](https://superuser.com/a/623305/160542) and
[stackoverflow](http://stackoverflow.com/a/13997892/909579) threads.


### Skipping selected repositories

pretty-git-prompt asks libgit2 for the repository status every time your prompt
is rendered. In huge repositories (linux, netbsd-src, kubernetes, ...) this can
take seconds, which makes the shell feel sluggish.

There is no ignore list inside the config file: the decision is made in your
shell, by wrapping the call in a function which prints nothing for the paths
you don't care about. Both snippets below match a directory and everything
below it, so subdirectories of the repository are skipped as well.

#### zsh

```shell
# Directory trees where pretty-git-prompt should stay quiet.
PGP_IGNORED_PATHS=(
  ~/dev/linux
  ~/dev/netbsd-src
)

pretty_git_prompt_unless_ignored() {
  local ignored
  for ignored in $PGP_IGNORED_PATHS; do
    [[ $PWD == $ignored || $PWD == $ignored/* ]] && return 0
  done
  pretty-git-prompt
}

setopt PROMPT_SUBST
RPROMPT='$(pretty_git_prompt_unless_ignored)'
```

#### bash

```shell
PGP_IGNORED_PATHS=(
  "$HOME/dev/linux"
  "$HOME/dev/netbsd-src"
)

pretty_git_prompt_unless_ignored() {
  local ignored
  for ignored in "${PGP_IGNORED_PATHS[@]}"; do
    if [[ $PWD == "$ignored" || $PWD == "$ignored"/* ]]; then
      return 0
    fi
  done
  pretty-git-prompt
}

pretty_prompt() { PS1="$(pretty_git_prompt_unless_ignored)\n\$ "; }
export PROMPT_COMMAND="pretty_prompt ; $PROMPT_COMMAND"
```

If you prefer to mark the repositories themselves instead of listing them in
your shell config, put a marker file in the repository, e.g.
`touch ~/dev/linux/.git/pretty-git-prompt-ignore`, and check for it instead:

```shell
pretty_git_prompt_unless_ignored() {
  local git_dir
  git_dir=$(git rev-parse --absolute-git-dir 2>/dev/null) || return 0
  [[ -e $git_dir/pretty-git-prompt-ignore ]] && return 0
  pretty-git-prompt
}
```

Before you exclude a repository, it may be worth speeding git itself up, since
pretty-git-prompt is as fast as the status of the repository it inspects:

```
$ git config core.untrackedCache true
$ git config core.fsmonitor true
```


## Configuration

The configuration is documented inside default config file. Therefore it's not
explicitly written down here. You can obtain it via:

```
$ pretty-git-prompt create-default-config
Configuration file created at "/home/you/.config/pretty-git-prompt.yml"
```

This repository contains also configuration for bash and zsh with colors:

1. [`files/pretty-git-prompt.yml.bash`](https://github.com/TomasTomecek/pretty-git-prompt/blob/master/files/pretty-git-prompt.yml.bash)
2. [`files/pretty-git-prompt.yml.zsh`](https://github.com/TomasTomecek/pretty-git-prompt/blob/master/files/pretty-git-prompt.yml.zsh)

In case anything is not clear from the comments inside the config files, please
open a new issue.

These are the values you can put in the `values` list, every one of them
formatted with your own `pre_format` and `post_format`:

| `type` | Displays |
| --- | --- |
| `repository_state` | state of the repository when it is not clean: merge, rebase, cherry-pick, ... |
| `remote_difference` | a remote branch and how far the local branch is ahead/behind it, with `name`, `ahead` and `behind` as nested `values` |
| `tag` | name of a tag pointing exactly at the checked out commit |
| `new` | number of untracked files |
| `changed` | number of tracked files changed in the working tree |
| `staged` | number of files added to the index |
| `conflicts` | number of conflicting files |
| `stash` | number of items in the stash |
| `separator` | a delimiter, either `display: always` or `display: surrounded` (shown only when there is a value displayed on every side it has) |

The branch name is part of `remote_difference`: its `name` value substitutes
`<LOCAL_BRANCH>`, `<REMOTE>`, `<REMOTE_BRANCH>` and `<REMOTE_FIRST_LETTER>`
(which falls back to `no_remote_placeholder`, `_` by default, when the branch
has no remote counterpart). Omit `remote_branch` to follow the tracked branch,
or set it to e.g. `upstream/master` to watch divergence against an arbitrary
branch.


### Where the config file lives

The config is read from `$XDG_CONFIG_HOME/pretty-git-prompt.yml`, which is
`~/.config/pretty-git-prompt.yml` unless you set `XDG_CONFIG_HOME` yourself.
When the file is not there, the default config (the one
`create-default-config` writes) is used, so the tool works before you configure
anything.

A config file elsewhere can be used with `--config`:

```
$ pretty-git-prompt --config ./my-prompt.yml
```


### Picking colors

`pre_format` and `post_format` expect prompt escapes of your shell, which a
terminal does not render on its own. These two subcommands translate them, so
you can see the result without touching your shell config:

```
$ pretty-git-prompt list-colors
```

prints every color and text style rendered in your terminal, next to the
snippet to paste into the config file:

```
        color                zsh
  ████  blue (4)             %{%F{blue}%}…%{%f%}
```

The list ends with the 256 color palette, so you can pick a number for
`%{%F{166}%}` or `\[\e[38;5;166m\]`.

```
$ pretty-git-prompt preview
```

renders your config for the current repository. With `--demo` it renders made
up repository states instead, which is handy while editing the config outside
of a repository, or to see values you rarely hit:

```
$ pretty-git-prompt preview --demo
  clean repository                     master
  new, changed and staged files        master│✚3Δ2▶1
  diverged from the remote branch      feature↑2↓1│upstream↑2↓1
  branch without a remote counterpart  feature
  tag pointing at HEAD                 master│#0.3.0
  conflict during a merge              merge│master│Δ1✖1
  stashed changes                      master│☐2
  detached HEAD                        2a4b9c1│Δ1
```

Both commands guess the shell from the config file, then from `$SHELL`; use
`--shell bash` or `--shell zsh` to override it. `--no-color` (or the `NO_COLOR`
environment variable) strips the formatting and prints the plain text.


## Command line interface

Run without arguments, the tool prints the prompt for the repository in the
current working directory — this is the only thing your shell config needs.
The rest is there to set it up and to debug it:

| Command | What it does |
| --- | --- |
| `pretty-git-prompt` | print the prompt for the current repository, nothing when it is not a git repository |
| `pretty-git-prompt --config FILE` | use `FILE` instead of the config in `$XDG_CONFIG_HOME` |
| `pretty-git-prompt --debug` | print what the tool is doing while it renders the prompt |
| `pretty-git-prompt create-default-config` | write the documented default config to `$XDG_CONFIG_HOME/pretty-git-prompt.yml` |
| `pretty-git-prompt list-colors` | list colors and text styles with the codes to put in the config file |
| `pretty-git-prompt preview` | render your config for the current repository |
| `pretty-git-prompt preview --demo` | render your config for made up repository states |

`list-colors` and `preview` accept `--shell bash|zsh` and `--no-color`,
`preview` accepts `--config` as well.


## Solving problems

If you encounter a problem, you may run the tool with verbose output to help you resolve the issue:

```
$ pretty-git-prompt --debug
Debug messages are enabled.
This is not a git repository: Error { code: -3, klass: 6, message: "could not find repository from \'.\'" }
```


## Contributing

This is an open source project. I don't guarantee any support. Everything is best effort.

If you encounter any issue, please submit it! I will take a look. The best
thing to do in the meanwhile is to try fixing it yourself.

The whole development environment should be trivial to setup, even run tests:
all you need is `podman` and `make`.

```
$ make test
```

[CONTRIBUTING.md](./CONTRIBUTING.md) describes the repository layout, both test
suites, the CI (Fedora builds and tests via [packit](https://packit.dev/)) and
how a release is made — please read it before you start hacking.


## Credits

This tool is heavily inspired by
[zsh-git-prompt](https://github.com/olivierverdier/zsh-git-prompt). At some
point I realized, I wanted a more powerful tool so I wrote pretty-git-prompt.
