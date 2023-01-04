import click
import zipfile
from pathlib import Path

import pkg_resources
import json

import shutil
import os


config = json.load(pkg_resources.resource_stream("blatex", 'resources/config.json'))
templatedir = Path(pkg_resources.resource_filename("blatex", "resources/templates"))

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
    os.system(f"{git} init")
    os.system(f"{git} add {str(directory)!r}")
    os.system(f"{git} commit -a -m 'blatex init'")
    


# ====================================== INTERFACE ====================================== 
@click.command("init")
@click.option('-t', '--template', "template", help="Name of the templates to use.")
@click.option('-d', '--dir', 'directory', type=click.Path(exists=True), help="Directory to initialize latex project in.")
@click.option('--no-git', is_flag=True, default=False, help="If set: does not create a git repo in the project directory.")
def blatex_init(template, directory, no_git):
    """Command for initializing a latex project"""

    if template in [t.stem for t in templatedir.iterdir()]:
        template = templatedir / f"{template}.zip"
    else:
        if template:
            click.echo(f"There is no template with the name {template!r}.\n")
        template = choose_template()
    
    
    if not directory:
        directory = Path.cwd()
    if not isinstance(directory, Path):
        directory = Path(directory)

    copy_template(template, directory)

    if not no_git:
        init_git_repo(directory)


@click.command("templates")
def list_templates():
    """List available templates"""
    for template in templatedir.iterdir():
        click.echo(template.stem)

@click.group("list")
def blatex_list():
    """Group of commands to list things in blatex"""
    pass

blatex_list.add_command(list_templates)
    

@click.group()
def blatex():
    """Cli for managing latex projects"""
    pass


blatex.add_command(blatex_init)
blatex.add_command(blatex_list)
