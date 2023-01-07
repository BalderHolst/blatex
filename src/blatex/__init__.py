import click

import zipfile
from pathlib import Path

import pkg_resources
import json
import re

import shutil
import subprocess
import os

from blatex.log_parser import *

local_config_file_name = ".blatex"

builtin_template_dir = Path(pkg_resources.resource_filename("blatex", "resources/templates"))

def get_root_dir(current_directory: Path | None = None, i: int = 0) -> Path:
    if i > 100:
        click.echo("Not inside initialized latex project. Initialize the current directory with `blatex init`.")
        exit(1)
    if current_directory == None:
        current_directory = Path.cwd()
    if local_config_file_name in [f.name for f in current_directory.iterdir()]:
        return(current_directory)
    return(get_root_dir(current_directory.parent, i + 1))
    

def get_global_config(config: str):
    """Return the global config. This looks in '~/.config/blatex/config.json' for user specific configuration."""

    user_config_file = Path("~/.config/blatex/config.json").expanduser()

    if user_config_file.exists():
        user_config = json.load(user_config_file.open())

        if config in user_config:
            return(user_config[config])

    default_config = json.load(open(pkg_resources.resource_filename("blatex", "resources/default-global-config.json")))

    return(default_config[config])

def get_local_config(config: str):
    local_config_file = get_root_dir() / local_config_file_name
    return(json.load(local_config_file.open())[config])

def get_cmd(cmd_name):
    cwd = Path.cwd()

    os.chdir(get_root_dir())

    cmd = get_local_config(cmd_name).replace(get_local_config('main-file-placeholder'), '"' + get_local_config('main-file') + '"') # TODO does not work with spaces

    os.chdir(cwd)

    return(cmd)

def get_templates():

    templates = []

    if get_global_config('enable_builtin_templates'):
        templates.extend(list(builtin_template_dir.iterdir()))

    for d in get_global_config('custom_template_dirs'):
        d = Path(d).expanduser()
        if not d.exists():
            click.echo(f"Templates directory: {str(d)!r} does not exist.")
        templates.extend(list(d.iterdir()))

    return(templates)


def choose_template():
    templates = get_templates()

    click.echo("Choose a template:")
    for n, template in enumerate(templates):
        click.echo("\t" + str(n) + ": " + template.stem)

    while True:
        nr = click.prompt("Template index")
        try:
            nr = int(nr)
        except ValueError:
            click.echo(f"{nr!r} is not a value template index.\n")
            continue
        if nr < len(templates) and nr >= 0:
            break
        click.echo(f"You must input a number between 0 and {len(templates) - 1}\n")

    return(templates[nr])

        
def copy_template(templatefile: Path | str, destination: Path | str):

    if not re.search(r".+\.zip$", str(templatefile)):
        click.echo(f"Template file {str(templatefile)!r} is not a valid template. It is not a '.zip' file.")
        exit(1)

    with zipfile.ZipFile(templatefile, mode="r") as archive:
         archive.extractall(destination)

def add_local_config_file(directory: Path, verbose=False):

    if local_config_file_name in directory.iterdir():
        if verbose:
            click.echo(f"No need to initialize {local_config_file_name!r} as it already exists.")
        return

    if verbose:
        click.echo("Using default configuration file.")

    config_file = Path(pkg_resources.resource_filename("blatex", "resources/default-local-config.json"))
    shutil.copy(config_file, f"{directory}/{local_config_file_name}")

    click.echo(f"Added config file {local_config_file_name!r} to the root dir.")

def get_installed_packages(search_dir = None): # TODO return package-module tree
    packages = []
    if not search_dir:
        search_dir = Path(get_global_config("package-install-location"))
    for file in search_dir.iterdir():
        if re.search(r".+\.sty", file.name):
            packages.append(file.stem)
        elif file.is_dir():
            packages.extend(get_installed_packages(file))
    return(packages)

def find_packages_in_file(file: Path):
    try:
        return(re.findall(r"usepackage{(\w+)}", file.read_text()))
    except UnicodeDecodeError:
        return []

def get_used_packages(directory=None):
    packages = []
    if not directory:
        directory = get_root_dir()
    for file in directory.iterdir():
        if file.is_file():
            packages.extend(find_packages_in_file(file))
        elif file.is_dir():
            packages.extend(get_used_packages(file))
    return(list(set(packages)))

def has_git():
    if shutil.which("git"):
        return(True)
    return(False)

# TODO test on windows
def init_git_repo(directory: Path):
    if not has_git():
        return
    
    click.echo("\nInitialising Git Repo:")
    git = f"git -C {str(directory)!r}"
    subprocess.run(f"{git} init", shell=True)
    subprocess.run(f"{git} add {str(directory)!r}", shell=True)
    subprocess.run(f"{git} commit -a -m 'blatex init'", shell=True)



def echo_errors(echo_logs = False, color=True):


    log_file = get_root_dir() / (Path(get_local_config('main-file')).stem + ".log")

    if not log_file.exists():
        click.echo("No log file found.")
        return

    (warnings, errors) = parse_log_file(log_file, echo_logs=echo_logs)

    if len(errors) == 0 and len(warnings) == 0:
        if color:
            click.echo(colored("Finished with no errors!", "green"))
        else:
            click.echo("Finished with no errors!")

    for warning in warnings:
        warning.echo(color=color)
        click.echo()

    for error in errors:
        error.echo(color=color)
        click.echo()

    # click.echo("=" * WIDTH)
    # click.echo(f"For more details, run blatex list errors --log, or check the {str(log_file.relative_to(Path.cwd()))!r} file manually.")


# ====================================== INTERFACE ====================================== 

CONTEXT_SETTINGS = dict(help_option_names=['-h', '--help'])


@click.command("init", context_settings=CONTEXT_SETTINGS)
@click.option('-t', '--template', "template", help="Name of the templates to use.")
@click.option('-d', '--dir', 'directory', type=click.Path(exists=True), help="Directory to initialize latex project in.")
@click.option('--git', is_flag=True, default=False, help="Initialize git repo in the root directory.")
def blatex_init(template, directory, git):
    """Initialize a latex project"""

    if not directory:
        directory = Path.cwd()
    if not isinstance(directory, Path):
        directory = Path(directory)

    if len(list(directory.iterdir())) > 0: # is the directory is not empty
        if (directory / local_config_file_name).exists():
            click.echo("Nothing to do.")
            return
        add_local_config_file(directory)
        return

    templates = get_templates()
    template_names = [t.stem for t in templates]

    if template in template_names: # --template option 
        template = templates[template_names.index(template)]
    elif template: # Wrong --template option
        click.echo(f"There is no template with the name {template!r}.")
        exit(1)
    else:
        template = choose_template()
    
    copy_template(template, directory)

    add_local_config_file(directory)

    if git:
        init_git_repo(directory)


@click.command("templates", context_settings=CONTEXT_SETTINGS)
@click.option("--full-path", "full_path", is_flag=True, help="Print the full path to the templates.")
def blatex_list_templates(full_path):
    """List available templates"""
    if full_path:
        for template in get_templates():
            click.echo(str(template))
        return

    for template in get_templates():
        click.echo(template.stem)

@click.command("errors", context_settings=CONTEXT_SETTINGS)
@click.option("--log", is_flag=True, help="Stylishly print the log file.")
@click.option("--no-color", "no_color", is_flag=True, help="Disable colored output.")
def blatex_list_errors(log, no_color):
    """List errors and warnings from last time the document was compiled."""
    echo_errors(echo_logs=log, color=not no_color)

@click.command("packages", context_settings=CONTEXT_SETTINGS)
@click.option("--no-color", "no_color", is_flag=True, help="Disable colored output.")
@click.option("--needed", is_flag=True, help="List only packages that are not installed, but used in the document.")
def blatex_list_packages(no_color, needed):
    """List packages used in the project"""
    installed_packages = get_installed_packages()
    used_packages = get_used_packages()
    
    used_packages.sort()

    for package in used_packages:
        if package in installed_packages:
            if needed:
                continue
            if not no_color:
                click.echo(colored(f"{package} [INSTALLED]", "green"))
            else:
                click.echo(f"{package} [INSTALLED]")
        else:
            if not no_color:
                if needed:
                    click.echo(colored(f"{package}", "red"))
                    continue
                click.echo(colored(f"{package} [NOT INSTALLED]", "red"))
            else:
                click.echo(package)

@click.command("installed-packages", context_settings=CONTEXT_SETTINGS)
def blatex_list_installed_packages():
    """List installed packages"""
    for package in get_installed_packages():
        click.echo(package)

@click.group("list", context_settings=CONTEXT_SETTINGS)
def blatex_list():
    """Commands to list things in blatex"""
    pass

blatex_list.add_command(blatex_list_templates)
blatex_list.add_command(blatex_list_errors)
blatex_list.add_command(blatex_list_packages)
blatex_list.add_command(blatex_list_installed_packages)

@click.command("create", context_settings=CONTEXT_SETTINGS)
@click.option("-f", "--force", is_flag=True, help="Override current configuration.")
def blatex_config_global_create(force):
    """Create default config file ̈́in '~/.config/blatex'."""
    config_dir = Path("~/.config/blatex").expanduser()

    if not config_dir.exists():
        config_dir.mkdir()

    config_file = config_dir / "config.json"

    if config_file.exists() and not force:
        click.echo(f"{str(config_file)!r} already exists.")
        return

    default_config_file = Path(pkg_resources.resource_filename("blatex", "resources/default-global-config.json"))

    shutil.copy(default_config_file, config_file)

    templates_dir = (config_dir / "templates") # TODO get from config

    if not templates_dir.exists():
        templates_dir.mkdir()

@click.group("global", context_settings=CONTEXT_SETTINGS)
def blatex_config_global():
    pass

blatex_config_global.add_command(blatex_config_global_create)

@click.group("config", context_settings=CONTEXT_SETTINGS)
def blatex_config():
    """Commands for configuration"""
    pass

blatex_config.add_command(blatex_config_global)

@click.command("compile", context_settings=CONTEXT_SETTINGS)
@click.option('-v', '--verbose', is_flag=True, help='Be verbose.')
def blatex_compile(verbose=False):
    """
    Compile the document as specified by the config file.

    The config file '.blatex' can be found in the root directory next to the main .tex file.
    """
    cmd = get_cmd('compile-cmd')

    if verbose:
        click.echo(f"Running: {cmd!r}")

    subprocess.run(cmd.split(" "))

    click.echo("\n\n")
    echo_errors()

@click.command("clean", context_settings=CONTEXT_SETTINGS)
@click.option('-v', '--verbose', is_flag=True, help='Be verbose.')
def blatex_clean(verbose=False):
    """Clean temporary files from root directory."""

    cmd = get_cmd("clean-cmd")

    if verbose:
        click.echo(f"Running: {cmd!r}")

    subprocess.run(cmd.split(" ")) # TODO go to root first
    

@click.group(context_settings=CONTEXT_SETTINGS)
def blatex():
    """Cli for managing latex projects"""
    pass


blatex.add_command(blatex_init)
blatex.add_command(blatex_compile)
blatex.add_command(blatex_clean)
blatex.add_command(blatex_list)
blatex.add_command(blatex_config)
