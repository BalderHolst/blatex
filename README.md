# What is this?
This is a cli tool for managing latex projects. 

### Features
- `blatex init` : Initialize latex document with templates
- `blatex compile` : Compile from any sub-directory of the project.
- `blatex clean` : Clean temporary files from any sub-directory.
- `blatex packages` : List packages used by the document, and mark if they are installed. It will also recommend texlive packages that contain the missing tex package.
- `blatex errors` : Parse errors and warnings, and display them nicely.
- `blatex templates` : list available templates and their sources.

# Dependencies
The default latex engine is [latexmk](https://mg.readthedocs.io/latexmk.html). This can however be altered by editing the commands in the '.blatex' file in the root directory (this file is generated when initializin the project).

Package functionality requires the use of texlive for package management.

This package has only been tested on linux, but there are plans to get it workning on windows.

# Getting Started

Make sure you have `latexmk` or another latex compiler installed. This package works with `latexmk` out of the box so it is recomended. 

Run the following in any directory:

```bash
blatex init
```

This is now root of your latex project. If the directory is empty, you will be prompted to use a template.

Compile the document by running this command from any sub-directory of the project:

```bash
blatex compile
```

All features of every command or command group can be found by passing the '-h' flag.

# Configuration files

blatex uses two separate configuration files. One local to the current document and one global to the entire system.

## Local Configuration File
Upon initializing a project, a hidden file with the name '.blatex' will appear in the project root. This file is used by blatex as a config file, but also to locate the root directory of the project.

This file contains json data for configuring blatex in this project specifically.

Here are the settings:
| Option                     | Description                          |
|----------------------------|--------------------------------------|
| main-file                  | The main '.tex' file of the project. |
| main-file-placeholder      | The string that will automatically be replaced by the main file path in the other options. |
| compile-cmd                | The command used for compiling the document. Change this if you are not using latexmk. |
| clean-cmd                  | The command used for cleaning up temporary files in the root directory. |


## Global Configuration File
This file contains the global configuration of blatex and contains setting regarding custom templates and among other things.

This file is found here: '~/.config/blatex/config.json'.

If you have not made your own yet, you can simply run the following command:

```bash
blatex config global create 
```

This will create a global configuration file with the default configuration.

Here is a list of global options:
| Option                     | Description                          |
|----------------------------|--------------------------------------|
| enable_builtin_templates   | Enable the templates built into the blatex pip package |
| custom_template_dirs       | List of directories to look for templates ('.zip' files) in |
| package-install-location   | Location where texlive packages are installed. This is used in the `blatex list packages` command. |
