import click
import zipfile
from pathlib import Path

# TODO install this to a better place
templatedir = Path("~/Projects/blatex/templates").expanduser()

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

# @click.option('-d', '--dir', required=False, type=click.Path(exists=True), help="The dir to initialize the project in.")

@click.command("init")
@click.option('-t', '--template', "template", help="Name of the templates to use.")
@click.option('-d', '--dir', 'directory', type=click.Path(exists=True), help="Directory to initialize latex project in.")
def blatex_init(template, directory):
    """Command for initializing a latex project"""

    if not template in [t.stem for t in templatedir.iterdir()]:
        if template:
            click.echo(f"There is no template with the name {template!r}.\n")
        template = choose_template()
    
    if not directory:
        directory = Path.cwd()

    copy_template(template, directory)

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


if __name__ == "__main__":
    blatex_init()

