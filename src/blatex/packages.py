import blatex

from sqlite_integrated import Database, Column
import pkg_resources
import click
from termcolor import colored

def get_db() -> Database:
    db_file = pkg_resources.resource_filename("blatex", "resources/packages.db")
    return Database(db_file, silent=True)

def find_tex_packages(db: Database, texlive_package):

    sql = """SELECT * FROM texlive_packages tl
    JOIN texlive_to_tex ttt ON ttt.texlive_package_id = tl.id
    JOIN tex_packages t ON t.id = ttt.tex_package_id
    WHERE tl.name = """ + f"'{texlive_package}'"

    db.cursor.execute(sql)

    return db.cursor.fetchall()

def get_number_of_tex_packages(db: Database, texlive_package):
    return len(find_tex_packages(db, texlive_package))


def find_texlive_packages(db: Database, tex_package: str):

    sql = """SELECT tl.* FROM tex_packages t
    JOIN texlive_to_tex ttt ON ttt.tex_package_id = t.id
    JOIN texlive_packages tl ON tl.id = ttt.texlive_package_id
    WHERE t.name = """ + f"'{tex_package}'"

    db.cursor.execute(sql)

    return db.cursor.fetchall()

def echo_search(package_name):
    db = get_db()

    click.echo(" Tex packages ".center(blatex.OUTPUT_WIDTH, "="))
    
    search = db.SELECT().FROM("tex_packages").WHERE("name").LIKE("%" + package_name + "%").run()

    for p in search:
        click.echo(p['name'])

    exact = db.SELECT().FROM("tex_packages").WHERE("name", package_name).run()

    if exact:
        click.echo(colored("\nFound exact match: '" + exact[0]['name'] + "'!", "green"))


    click.echo("\n" + "Texlive packages ".center(blatex.OUTPUT_WIDTH, "="))
    
    search = db.SELECT().FROM("texlive_packages").WHERE("name").LIKE("%" + package_name + "%").run()

    for p in search:
        click.echo(p['name'])

    exact = db.SELECT().FROM("texlive_packages").WHERE("name", package_name).run()

    if exact:
        click.echo(colored("\nFound exact match: '" + exact[0]['name'] + "'!", "green"))

def echo_texlive_recommendations(tex_package, count=8, no_common=False):
    db = get_db()

    sql = """select tl.name, tl.contrib from tex_packages tp
join tex_to_texlive ttt on ttt.tex_package_id = tp.id
join texlive_packages tl on ttt.texlive_package_id == tl.id
where tp.name = """ + "\"" + str(tex_package) + "\""

    db.cursor.execute(sql)

    texlive_packages = db.cursor.fetchall()[:count]

    sql = """select mt.name from tex_packages tp
join tex_to_miktex ttt on ttt.tex_package_id = tp.id
join miktex_packages mt on ttt.miktex_package_id == mt.id
where tp.name = """ + "\"" + str(tex_package) + "\""

    db.cursor.execute(sql)

    miktex_packages = db.cursor.fetchall()[:count]

    if len(texlive_packages) == 0:
        click.echo(colored(f"Could not find any texlive package including tex package {tex_package!r}", "red"))
        return

    if len(miktex_packages) == 0:
        click.echo(colored(f"Could not find any miktex package including tex package {tex_package!r}", "red"))
        return

    click.echo(f"Recommended packages that include \'{colored(tex_package)}\':")

    badge = "    "
    for p in texlive_packages:

        if p[1]:
            click.echo(colored(badge + p[0] + " [texlive contrib]", "cyan"))
        else:
            click.echo(colored(badge + p[0] + " [texlive]", "yellow"))

    for p in miktex_packages:
        click.echo(colored(badge + p[0] + " [miktex]", "magenta"))
