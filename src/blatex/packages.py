from sqlite_integrated import Database, Column
import pkg_resources
import click
from termcolor import colored

def get_db() -> Database:
    db_file = pkg_resources.resource_filename("blatex", "resources/packages.db")
    return Database(db_file)

def find_tex_packages(db: Database, texlive_package):

    sql = """SELECT * FROM texlive_packages tl
    JOIN texlive_to_tex ttt ON ttt.texlive_package_id = tl.id
    JOIN tex_packages t ON t.id = ttt.tex_package_id
    WHERE tl.name = """ + f"'{texlive_package}'"

    db.cursor.execute(sql)

    return db.cursor.fetchall()

def get_number_of_tex_packages(db: Database, texlive_package):

    sql = """SELECT COUNT(*) FROM texlive_packages tl
    JOIN texlive_to_tex ttt ON ttt.texlive_package_id = tl.id
    JOIN tex_packages t ON t.id = ttt.tex_package_id
    WHERE tl.name = """ + f"'{texlive_package}'"

    db.cursor.execute(sql)

    return db.cursor.fetchall()[0][0]


def find_texlive_packages(db: Database, tex_package: str):

    sql = """SELECT tl.* FROM tex_packages t
    JOIN texlive_to_tex ttt ON ttt.tex_package_id = t.id
    JOIN texlive_packages tl ON tl.id = ttt.texlive_package_id
    WHERE t.name = """ + f"'{tex_package}'"

    db.cursor.execute(sql)

    return db.cursor.fetchall()

def echo_texlive_recommendations(tex_package, count=8, no_common=False):
    db = get_db()

    sql = """SELECT tl.* FROM tex_packages t
    JOIN texlive_to_tex ttt ON ttt.tex_package_id = t.id
    JOIN texlive_packages tl ON tl.id = ttt.texlive_package_id
    WHERE t.name = """ + "\"" + str(tex_package) + "\" " +  "ORDER BY common DESC, nr_of_tex_packages ASC"

    db.cursor.execute(sql)

    texlive_packages = db.cursor.fetchall()[:count]


    click.echo(f"Recommended texlive packages that include \'{colored(tex_package, 'red')}\'.")
    for p in texlive_packages:
        
        badge = "    "
        if p[3] and not no_common:
            badge = "(c) "

        click.echo(colored(badge + p[1], "yellow"))


