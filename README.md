# Introduction

Frob is a collection of useful tools for providing a good experience on minimal desktop environments (e.g. i3).

# Installation

```
cargo install frob --git https://github.com/panicbit/frob
```

You can also install any direct subcommand of `frob` as standalone tool by substituting `frob` in the above command with `frob-<subcommand>`.
E.g. to install `frob volume` as standalone tool use:

```
cargo install frob-volume --git https://github.com/panicbit/frob
```

See below for an overview of possible subcommands.

# Subcommand Overview

```
frob
    brightness
        up
        down
        get
        set
        list
    monitor
        cycle
    volume
        up
        down
        get
        set
        toggle-mute
```

The first level of `frob` subcommands can also be installed as individual tools e.g. `frob-volume`.
