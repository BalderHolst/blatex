import click

import zipfile
from pathlib import Path

import pkg_resources
import json
import re

import shutil
import subprocess

from blatex.log_parser import *

config_file_name = ".blatex"

templatedir = Path(pkg_resources.resource_filename("blatex", "resources/templates"))

def get_root_dir(current_directory: Path | None = None, i: int = 0) -> Path:
    if i > 100:
        raise RecursionError(f"Reached max iteration looking for the root dir. The root dir is defined af containing a {config_file_name!r} file.")
    if current_directory == None:
        current_directory = Path.cwd()
    if config_file_name in [f.name for f in current_directory.iterdir()]:
        return(current_directory)
    return(get_root_dir(current_directory.parent, i + 1))
    

def get_configs():
    return(json.load(open(get_root_dir() / config_file_name)))

def get_cmd(cmd_name):
    configs = get_configs()

    cmd = configs[cmd_name].replace(configs['main-file-placeholder'], str(get_root_dir() / configs['main-file']))

    return(cmd)

def choose_template():
    templates = [[f.stem, f] for f in templatedir.iterdir()]

    click.echo("Choose a template:")
    for n, template in enumerate(templates):
        click.echo("\t" + str(n) + ": " + str(template[0]))

    while True:
        nr = click.prompt("Template index: ")
        try:
            nr = int(nr)
        except ValueError:
            click.echo(f"{nr!r} is not a valus template index.")
            continue
        if nr < len(templates) and nr >= 0:
            break
        click.echo(f"You must input a number between 0 and {len(templates) - 1}")

    return(templates[nr][1])

        
def copy_template(templatefile: Path | str, destination: Path | str):
    with zipfile.ZipFile(templatefile, mode="r") as archive:
         archive.extractall(destination)

def add_config_file(directory: Path, verbose=False):

    if config_file_name in directory.iterdir():
        if verbose:
            click.echo(f"No need to initialize {config_file_name!r} as it already exists.")
        return

    if verbose:
        click.echo("Using default configuration file.")

    config_file = Path(pkg_resources.resource_filename("blatex", "resources/config.json"))
    shutil.copy(config_file, f"{directory}/{config_file_name}")

    click.echo(f"Added config file {config_file_name!r} to the root dir.")

def get_installed_packages(search_dir = None):
    packages = []
    if not search_dir:
        search_dir = Path("/usr/share/texlive/texmf-dist/tex/latex")
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


    log_file = get_root_dir() / (Path(get_configs()['main-file']).stem + ".log")

    if not log_file.exists():
        click.echo("No log file found.")
        return

    (warnings, errors) = parse_log_file(log_file, echo_logs=echo_logs)

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

    if len(list(directory.iterdir())) > 0:
        if (directory / config_file_name).exists():
            click.echo("Nothing to do.")
            return
        add_config_file(directory)
        return

    if template in [t.stem for t in templatedir.iterdir()]:
        template = templatedir / f"{template}.zip"
    else:
        if template:
            click.echo(f"There is no template with the name {template!r}.\n")
        template = choose_template()
    
    copy_template(template, directory)

    add_config_file(directory)

    if git:
        init_git_repo(directory)


@click.command("templates", context_settings=CONTEXT_SETTINGS)
def blatex_list_templates():
    """List available templates"""
    for template in templatedir.iterdir():
        click.echo(template.stem)

@click.command("errors", context_settings=CONTEXT_SETTINGS)
@click.option("--log", is_flag=True, help="Stylishly print the log file.")
@click.option("--no-color", "no_color", is_flag=True, help="Disable colored output.")
def blatex_list_errors(log, no_color):
    """List errors and warnings from last time the document was compiled."""
    echo_errors(echo_logs=log, color=not no_color)

@click.command("packages", context_settings=CONTEXT_SETTINGS)
@click.option("--no-color", "no_color", is_flag=True, help="Disable colored output.")
@click.option("--needed", is_flag=True, help="List only packages that are not installed.")
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
                click.echo(colored(package, "red"))
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

    subprocess.run(cmd.split(" "))
    

@click.group(context_settings=CONTEXT_SETTINGS)
def blatex():
    """Cli for managing latex projects"""
    pass


blatex.add_command(blatex_init)
blatex.add_command(blatex_compile)
blatex.add_command(blatex_clean)
blatex.add_command(blatex_list)
