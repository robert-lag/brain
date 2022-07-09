# Brain

Brain is a commandline tool for the organisation of a zettelkasten or as some people call it: **your second brain**.

![2022-02-03-131954](https://user-images.githubusercontent.com/61148783/152341690-98f3eea4-99dd-42b8-98c8-79d12f30b857.png)

Brain doesn't use a regular *Luhmann's Zettelkasten* but a slightly altered version. The differences are:
- Usage of timestamp as note ID
- Possibility of using different types of notes (can be ignored if you don't want to use it). More information about this explained in [Note types](#note-types)
    - Topic
    - Quote
    - Journal

Additionally it uses markdown-formatted notes with YAML-headers for metadata.

More information about the zettelkasten method:
- [zettelkasten.de/introduction](https://zettelkasten.de/introduction)
- [https://www.reddit.com/r/Zettelkasten/](https://www.reddit.com/r/Zettelkasten/)

## Contents

- [Features](#features)
- [Installation](#installation)
    - [Pre-built binaries](#pre-built-binaries)
    - [Cargo](#cargo)
- [Commands](#commands)
    - [Creating a new zettelkasten](#creating-a-new-zettelkasten)
    - [Adding notes](#adding-notes)
    - [Listing created notes](#listing-created-notes)
    - [Graph View](#graph-view)
    - [TUI Mode](#tui-mode)
        - [Keybindings in TUI Mode](#keybindings-in-tui-mode)
- [Note types](#note-types)
- [Note format](#note-format)
    - [Note format requirements](#note-format-requirements)
    - [Tags](#tags)
- [Note template](#note-template)
    - [Marker](#marker)
    - [Example: Note template](#example-note-template)
- [Searching for notes](#searching-for-notes)
    - [Search operators](#search-operators)
    - [Detailed description](#detailed-description)

## Features

Some of the most important features:

- **Vim**-like-keybindings in TUI
- **Automatic backlinking** of notes
- **Powerful search** functionality
- Notes are stored as simple **Markdown** files
- Your **Favorite editor** can be used for editing notes
- **Minimal** (non-bloated) software
- A **Graph View** you can view in your browser

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

## Commands

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

### Opening notes

Brain opens notes in the editor specified by the `EDITOR` environment variable. This makes it possible to use your favorite editor for editing your notes.

You can open a note either in the TUI mode or on the commandline:

~~~
brn open <note-id>
~~~

### Graph View

To view a graphical representation of the zettelkasten type:

~~~
brn graph
~~~

This will open a graph in the browser specified by the `BROWSER` environment variable.

![graph-view](https://user-images.githubusercontent.com/61148783/178108696-311f347d-08c9-4ada-acb8-b312b1f28ab0.png)

By clicking on a graph node you can see the name of the note the graph node represents.

![graph-view-node-selection](https://user-images.githubusercontent.com/61148783/178109252-b787c6f4-4c7d-49a9-9d87-9d3d67986f13.png)

It's also possible to select multiple nodes by pressing `Shift` and selecting nodes. This can be done either by pressing the nodes one by one or by using box selection (while pressing `Shift`).

![graph-view-multiple-selections](https://user-images.githubusercontent.com/61148783/178109250-ac7669de-4278-4260-98bb-783440857357.png)

### TUI mode

The TUI mode makes it easier to traverse thourgh your zettelkasten.

~~~
brn tui
~~~

#### Keybindings in TUI mode

| **Keys**           | **Description**                                        |
|--------------------|--------------------------------------------------------|
| `q`                | quit the program                                       |
| `j`, `UpArrow`     | up                                                     |
| `k`, `DownArrow`   | down                                                   |
| `l`, `LeftArrow`   | open note                                              |
| `g`                | go to the top of the note list                         |
| `G`                | go to the bottom of the note list                      |
| `h`                | show history of last visited notes                     |
| `r`                | show a list of random notes                            |
| `/`                | enter search mode                                      |    
| `ESC`              | show list of last created notes (default view)         |
| `a`                | add new note                                           |
| `x`                | remove currently selected note                         |
| `y`                | copy note link to currently selected note to clipboard |

## Note types

The idea of note types stems from this reddit (r/zettelkasten) and blog posts of the same author:
[The opposite collectors fallacy - Reddit](https://www.reddit.com/r/Zettelkasten/comments/gp3y2t/the_opposite_collectors_fallacy/)
[The opposite collectors fallacy - Blog post](https://technosoof.wordpress.com/2020/05/23/the-opposite-collectors-fallacy/)

This idea was integrated inside Brain by making it possible to create 3 different types of notes depending on the type of information inside them:
- **Topic** *(default)*
A standard zettelkasten note (= Zettel).
- **Quote**
Contains a quote which can then be analysed by topic notes.
- **Journal**
A note similar to a diary entry. Can then be analysed be topic notes.

Currently the type of note can only be set when creating new notes and cannot be changed afterwards.

The type of a note is reflected in the first character of the note ID. `T` stands for *Topic*, `J` for *Journal* and `Q` for *Quote*.

Examples:
- `T20220101120000`
- `Q20220101120000`
- `J20220101120000`

On the commandline you can specify the type of the note by using the correspondig flag:
- `brn add` creates a Topic-note
- `brn add -t` creates a Topic-note
- `brn add -q` creates a Quote-note
- `brn add -j` creates a Journal-note

## Note format

### Note format requirements

All notes should satisfy the following format requirements. If they don't, then errors could occur.

- The YAML header (= metadata) should always be on top of a note file
- The YAML header needs at least the following fields:
    - id
    - name
    - date
    - tags
    - backlinks
- Don't change the values of the following fields in the YAML-header (= metadata), as thery are filled automatically:
    - id
    - date
    - backlinks

All values of the metadata fields can be split into multiple lines:

~~~yaml
---
id: T20220101120000
name: This is a very long note name
      that was split into 2 lines
tags: [ my-first-tag, my-second-tag, my-third-tag,
        my-fourth-tag, my-fifth-tag ]
...
~~~

The order of the fields in the YAML header doesn't matter:
~~~yaml
---
name: This is a very long note name
      that was split into 2 lines
id: T20220101120000
tags: [ my-first-tag, my-second-tag, my-third-tag,
        my-fourth-tag, my-fifth-tag ]
...
~~~

### Tags

Tags can be declared by changing the value of the corresponding YAML header field. They are separated by colons:

~~~yaml
tags: [ my-first-tag, my-second-tag, my-third-tag ]
~~~

To be able to use spaces and special characters for tags you need to quote them:

~~~yaml
tags: [ "#my-first-tag", "my second tag", my-third-tag ]
~~~

**Important:** It doesn't matter if you declare your tags with a `#` or without as it will be ignored by the program. When searching for tags you can always use a `#` in the beginning of the search text, to make sure that only tags are searched and not note names. For more information about searching notes see [Searching for notes](#searching-for-notes)

## Note template

When executing `brn init` a hidden directory called `.zettelkasten/` is created in the project folder.

In there you will find a file called `note-template.md`. By changing this file you can determine how a newly created note will be structured.

**Important:** The YAML-Header (= metadata) should always be at the top of the file, as it wouldn't be found otherwise. For more information regarding note formatting see [Note format requirements](#note-format-requirements).

### Marker

Markers can be used to represent note-specific data in the note template. When creating a new note those markers will be replaced by their corresponding data.

| **Marker**        | **Description**                |
|-------------------|--------------------------------|
| `<note-id>`       | Inserts the note ID            |
| `<note-name>`     | Inserts the note name          |
| `<creation-date>` | Inserts the creation timestamp |

### Example: Note template 

`./.zettelkasten/note-template.md`:

~~~markdown
---

id: <note-id>
name: <note-name>
date: <creation-date>
tags: [ ]
backlinks: [ ]

---

# <note-name>

## References

## Quotes

## Sources

~~~

If a new note with the name `my new note` would be created on the 1.January 2022 on 12 a.m. it would look like this:

~~~markdown
---

id: T20220101120000
name: my new note
date: 2022-01-01 12:00:00
tags: [ ]
backlinks: [ ]

---

# my new note

## References

## Quotes

## Sources

~~~

## Searching for notes

### Search operators

| **Operator** | **Description**                                      |
|--------------|------------------------------------------------------|
| `&&`         | Combines 2 search strings                            |
| `!`          | Excludes notes including the following search string |
| `#`          | Applies the following search string only for tags    |

### Detailed description

Notes can be searched with the subcommand `brn search` or inside the TUI mode using the `/` shortcut:

~~~shell
$ brn search my-first
~~~

This can result in for example:

~~~shell
T20200629000001 random note name        #my-first-tag
T20200629000002 some other note     #my-first-tag
T20200629000003 some other random note     #my-first-tag-2
T20210629000004 my-first-interesting-note       #my-first-tag
T20210718000005 my-first-note
~~~

You can see that the first 3 results were found because of their tag `my-first-tag`. The fourth one was found because of its name and its tag. In this case the tag is also displayed. The last result was found only because of its name. As the tags of this note don't matter in this case they aren't displayed here either.

If you only want to search for tags, then you can put a `#` in front of the search text. Note that the search text must be quoted this time as the shell would recognise the search text as a comment otherwise:

~~~shell
$ brn search "#my-first"
~~~

This will result in:

~~~
T20200629000001 random note name        #my-first-tag
T20200629000002 some other note     #my-first-tag
T20200629000003 some other random note     #my-first-tag-2
T20210629000004 my-first-interesting-note       #my-first-tag
~~~

As you can see the note `my-first-note` isn't displayed anymore, as it doesn't have any tag that contains the text `my-first`.

You can also combine different search requirements with `&&`. Note that you need to quote the search text now not only because of the `#` but also because it now includes spaces:

~~~shell
$ brn search "#my-first && random"
~~~

This will result in:

~~~
T20200629000001 random note name        #my-first-tag
T20200629000003 some random note     #my-first-tag-2
~~~

As you can see now there are only results which have a tag containing `my-first` and either a tag, a note name or note ID containing `random`.

You can also filter the results based on things you don't want inside your results with `!`:

~~~shell
$ brn search "#my-first && !random"
~~~

This will result in:

~~~
T20200629000002 some other note     #my-first-tag
T20210629000004 my-first-interesting-note       #my-first-tag
~~~

Now all results that include `random` in their note name, note ID or in any of their tags aren't displayed.

The specified search text also always searches the note IDs. As the note ID contains the timestamp of its creation this can be very useful. For example if you want to see all notes that were created in July 2021:

~~~
$ brn search 202107
~~~

This will result in:

~~~
T20210718000005 my-first-note
J20210703000032 some journal written in july
~~~

You can also filter based on note type:

~~~
$ brn search T202107
~~~

This will result in:

~~~
T20210718000005 my-first-note
~~~

Note that the journal now doesn't appear as the note is not a Topic-note but a Journal-note.
