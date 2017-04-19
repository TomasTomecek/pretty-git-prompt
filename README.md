# pretty-git-prompt

[![Build Status](https://travis-ci.org/TomasTomecek/pretty-git-prompt.svg?branch=master)](https://travis-ci.org/TomasTomecek/pretty-git-prompt)

Your current git repository information inside a beautiful shell prompt.

![Preview of pretty-git-prompt](/data/example.png)


## Development status

The tool is ready for testing (I am using it for some time): most of the code
is already written, now I seek for feedback.

Since there was just a single release, it's very likely that configuration file
structure will change in future.


## How can I try this?

Very easily, actually! It just takes some time to prepare it (create build
environment, compile the tool, run the demo).

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

![]() TBD

This is an interactive shell, so you can play with it.


### bash

```
$ make bash-demo
```

![]() TBD

This demo is one of the ways I test the tool.


## Usage

If you want to add it inside your shell, this section contains information how to do that.


### Obtaining `pretty-git-prompt` binary


#### GitHub release

Get latest binary via GitHub release:

```
$ curl -O TBD
```


#### Compile it yourself

```
$ make build
```

As stated inside demo section above, this takes some time.

If you have rust compiler and cargo available on your system, you can compile
the tool without using a container:

```
$ cargo build --release
```

The binary is then available on this path:

```
$ ls -lha target/release/pretty-git-prompt
-rwxr-xr-x 2 user group 4.8M Apr  1 21:37 target/release/pretty-git-prompt
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
explicitely written down here. You can obtain it via:

```
$ pretty-git-prompt create-default-config
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
 4. Easy to contibute to:
    * Build with a single command.
    * Build inside predictive environment.
    * Test with a single command.

If you encounter any issue, please submit it! I will take a look. The best
thing to do in the meanwhile is to try fixing it yourself.

The whole development environment should be trivial to setup, even run tests.

All you need is [docker](https://github.com/docker/docker) engine running and `make`.

First you need to build container image with rust and all dependencies inside:

```
$ make unstable-environment
```

This is using latest usable nightly rust.

And then just make sure all tests are passing and you are not introducing any new warnings:

```
$ make test
```

If any of the two `make` invocations above doesn't work for you, please open an issue.


## Credits

This tool is heavily inspired by
[zsh-git-prompt](https://github.com/olivierverdier/zsh-git-prompt). At some
point I realized, I wanted a more powerful tool.
