import click
import zipfile
from pathlib import Path

import pkg_resources
import json

import shutil
import subprocess

config_file_name = ".blatex"

templatedir = Path(pkg_resources.resource_filename("blatex", "resources/templates"))

def find_root_dir(current_directory: Path | None = None, i: int = 0) -> Path:
    if i > 100:
        raise RecursionError(f"Reached max iteration looking for the root dir. The root dir is defined af containing a {config_file_name!r} file.")
    if current_directory == None:
        current_directory = Path.cwd()
    if config_file_name in [f.name for f in current_directory.iterdir()]:
        return(current_directory)
    return(find_root_dir(current_directory.parent, i + 1))
    

def get_configs():
    return(json.load(open(find_root_dir() / config_file_name)))

def get_cmd(cmd_name):
    configs = get_configs()

    cmd = configs[cmd_name].replace(configs['main-file-placeholder'], str(find_root_dir() / configs['main-file']))

    return(cmd)

def choose_template():
    templates = [[f.stem, f] for f in templatedir.iterdir()]

    print("Choose a template:")
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
    


# ====================================== INTERFACE ====================================== 

CONTEXT_SETTINGS = dict(help_option_names=['-h', '--help'])


@click.command("init", context_settings=CONTEXT_SETTINGS)
@click.option('-t', '--template', "template", help="Name of the templates to use.")
@click.option('-d', '--dir', 'directory', type=click.Path(exists=True), help="Directory to initialize latex project in.")
@click.option('--no-git', is_flag=True, default=False, help="If set: does not create a git repo in the project directory.")
def blatex_init(template, directory, no_git):
    """Command for initializing a latex project"""

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

    if not no_git:
        init_git_repo(directory)


@click.command("templates", context_settings=CONTEXT_SETTINGS)
def blatex_list_templates():
    """List available templates"""
    for template in templatedir.iterdir():
        click.echo(template.stem)

@click.group("list", context_settings=CONTEXT_SETTINGS)
def blatex_list():
    """Commands to list things in blatex"""
    pass

blatex_list.add_command(blatex_list_templates)

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
