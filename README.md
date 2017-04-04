# pretty-git-prompt

[![Build Status](https://travis-ci.org/TomasTomecek/pretty-git-prompt.svg?branch=master)](https://travis-ci.org/TomasTomecek/pretty-git-prompt)

Beautiful prompt with info about your current git repository.

![Preview of pretty-git-prompt](/data/example.png)

## Usage

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

Make sure that binary `pretty-git-prompt` is placed on your `$PATH`.


## Solving problems

If you encounter a problem, you may run the tool with verbose output to help you resolve the issue:

```
$ pretty-git-prompt --debug
Debug messages are enabled.
This is not a git repository: Error { code: -3, klass: 6, message: "could not find repository from \'.\'" }
```

## Configuration

## Contributing

## Credits
