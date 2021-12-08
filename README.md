# Introduction

Midas is a collection of useful tools for providing a good experience on minimal desktop environments (e.g. i3).

# Installation

```
cargo install midas --git https://github.com/panicbit/midas
```

You can also install any direct subcommand of `midas` as standalone tool by substituting `midas` in the above command with `midas-<subcommand>`.
E.g. to install `midas volume` as standalone tool use:

```
cargo install midas-volume --git https://github.com/panicbit/midas
```

See below for an overview of possible subcommands.

# Subcommand Overview

```
midas
    monitor
        cycle
    volume
        get
        increase
        decrease
```

The first level of `midas` subcommands can also be installed as individual tools e.g. `midas-volume`.