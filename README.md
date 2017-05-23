# pretty-git-prompt

[![Build Status](https://travis-ci.org/TomasTomecek/pretty-git-prompt.svg?branch=master)](https://travis-ci.org/TomasTomecek/pretty-git-prompt)

Your current git repository information inside a beautiful shell prompt.

![Preview of pretty-git-prompt](/data/example.png)

Features:

 * You are able to display values such as:
   * git repository state (resolving `merge` conflict, interactive `rebase`, ...)
   * Current branch name.
   * Count of changed, newly-added, staged, conflicting files.
 * You can track divergence against arbitrary branches.
 * Every value in output can be fully configured via a config file.
 * Sample configuration files feature colors.
 * The tool supports `zsh` and `bash`.
 * pretty-git-prompt is written in Rust programming language and is delivered as a single, statically-linked binary.


## Development status

The tool is ready to use.


## How can I try this out?

Very easily! You don't need to install pretty-git-prompt if you just want to
see it in action. There is a make target which launches docker container with
whole environment set up.

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


#### GitHub release

Get the binary via [latest GitHub release](https://github.com/TomasTomecek/pretty-git-prompt/releases/latest).

For a linux distrubution:

```
$ curl -O https://github.com/TomasTomecek/pretty-git-prompt/releases/download/0.1.2/pretty-git-prompt-0.1.2-x86_64-unknown-linux-gnu
```

Or for MacOS:

```
$ curl -O https://github.com/TomasTomecek/pretty-git-prompt/releases/download/0.1.2/pretty-git-prompt-0.1.2-x86_64-apple-darwin
```



#### Compile it yourself

```
$ make build
```

As stated inside demo section above, this takes some time.

If you have rust compiler and cargo available on your system, you can compile
the tool without using a container:

```
$ make exec-stable-build
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
master|✚1Δ1
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
RPROMPT='\$(pretty-git-prompt)'
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


## Solving problems

If you encounter a problem, you may run the tool with verbose output to help you resolve the issue:

```
$ pretty-git-prompt --debug
Debug messages are enabled.
This is not a git repository: Error { code: -3, klass: 6, message: "could not find repository from \'.\'" }
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


## Contributing

### Support

This is an open source project. I don't guarantee any support. Everything is best effort.


### Writing code

This project builds upon several principles:

 1. Configurable as much as possible.
 2. Pretty and useful.
 3. As few dependencies as possible.
 4. Easy to contribute to:
    * Build with a single command.
    * Build inside predictive environment.
    * Test with a single command.

If you encounter any issue, please submit it! I will take a look. The best
thing to do in the meanwhile is to try fixing it yourself.

The whole development environment should be trivial to setup, even run tests.

All you need is [docker](https://github.com/docker/docker) engine running and `make`.

First you need to build container image with rust and all dependencies inside:

```
$ make nightly-environment
```

This is using latest nightly rust. The nightly is used because of [clippy](https://github.com/Manishearth/rust-clippy).

And then just make sure all tests are passing and you are not introducing any new warnings:

```
$ make test
```

If any of the two `make` invocations above doesn't work for you, please open an issue.


## Credits

This tool is heavily inspired by
[zsh-git-prompt](https://github.com/olivierverdier/zsh-git-prompt). At some
point I realized, I wanted a more powerful tool so I wrote pretty-git-prompt.
