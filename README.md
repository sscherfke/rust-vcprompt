Informative version control prompt for your shell
=================================================

[![build status](https://gitlab.com/sscherfke/rust-vcprompt/badges/master/build.svg)](https://gitlab.com/sscherfke/rust-vcprompt/commits/master)

A small program that prints a summary of the current git/hg repository for use
in a shell prompt (like Bash or ZSH).

You can choose between two styles – one with full details and a minimal one:

![Full](vcprompt-full.png) ![Full](vcprompt-minimal.png)

This program is inspired by (can can be configured to be compatible with) the
[oh-my-zsh
git-prompt](https://github.com/robbyrussell/oh-my-zsh/tree/master/plugins/git-prompt)
and the original [vncprompt
C implementation](https://bitbucket.org/gward/vcprompt).
