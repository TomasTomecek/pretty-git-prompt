# pretty-git-prompt

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
RPROMPT='\$(pretty-git-prompt -m zsh)'
```

Make sure that binary `pretty-git-prompt` is placed on your `$PATH`.


## Configuration

## Contributing

## Credits
