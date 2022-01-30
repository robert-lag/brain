# Brain

Brain is a commandline tool for the organisation of a zettelkasten or as some call it: your second brain.

More information about the zettelkasten method:
- [zettelkasten.de/introduction](https://zettelkasten.de/introduction)
- [https://www.reddit.com/r/Zettelkasten/](https://www.reddit.com/r/Zettelkasten/)

## Contents

- [Installation](#installation)
    - [Pre-built binaries](#pre-built-binaries)
    - [Cargo](#cargo)
- [Usage](#usage)
    - [Creating a new zettelkasten](#creating-a-new-zettelkasten)
    - [Adding notes](#adding-notes)
    - [Listing created notes](#listing-created-notes)
    - [TUI Mode](#tui-mode)


## Installation

The binary executable is called `brn`.

### Pre-built binaries

In the Releases page (on Github) you can find pre-built binaries for every version. 

### Cargo

To build a binary using `cargo` go in the project root directory (the directory with `Cargo.toml` inside it) and execute:

~~~
cargo build --release
~~~

You can find the new binary in the path `./target/release/brn`.

## Usage

Brain can be used in 2 different ways:
- You can use it from the commandline using various subcommands like:
    - `brn add`
    - `brn remove`
    - `brn list`
    - ...
- You can use the TUI mode for a more interactive experience.
    - `brn tui`
    - With this you can do nearly everything you can do with all other subcommands. Exceptions:
        - `brn init`
        - `brn random`

For a full list of available commands type `brn help`.

### Creating a new zettelkasten

Create a new directory, where you want to create your zettelkasten and navigate into it:

~~~
mkdir my-zettelkasten
cd my-zettelkasten
~~~

Then execute:

~~~
brn init
~~~

The current directory will be recognized as a zettelkasten from now on.

### Adding notes

~~~
brn add "A Zettelkasten is great"
~~~

### Listing created notes

~~~
brn list
~~~

### TUI mode

The TUI mode makes it easier to traverse thourgh your zettelkasten.

~~~
brn tui
~~~
