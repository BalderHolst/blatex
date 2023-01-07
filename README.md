# What is this?
This is a cli tool for managing latex templates. 

### Features
- `blatex init` : Initialize latex document with templates
- `blatex compile` : Compiling from any sub-directory of the project.
- `blatex clean` : Clean temporary files from any sub-directory.
- `blatex list packages` : List packages used by the document, and mark if they are installed
- `blatex list errors` : Parse errors and warnings, and display them nicely - THIS IS NOT FULLY DONE

# Dependencies
The default latex engine is [latexmk](https://mg.readthedocs.io/latexmk.html). This can however be altered by editing the commands in the '.blatex' file in the root directory (this file is generated when initializin the project).

Package funktionality requires the use of texlive for package management.

This package has only been tested on linux.

# Getting Started

Make sure you have `latexmk` or another latex compiler installed. This package works with `latexmk` out of the box so it is recomended. 

Run the following in any directory:

```bash
blatex init
```

This is now root of your latex project. If the directory is empty, you will be prompted to use a template.

Compile the document by running this command from any sub-directory of the project.

```bash
blatex compile
```

All features of every command or command group can be found by passin the '--help' flag.
