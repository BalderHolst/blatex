import json
from pathlib import Path
import os
from sqlite_integrated import Database, Column, ForeignKey

import blatex.packages

db_file = Path("./src/blatex/resources/packages.db")

json_file = Path("scripts/find_package_names/packages.json")

def create_blank_database():

    if db_file.exists():
        os.remove(db_file)

    db = Database(db_file, new=True)

    db.create_table("texlive_packages", [
            Column("id", "integer", primary_key=True),
            Column("name", "string"),
            Column("nr_of_tex_packages", "int"),
            Column("common", "bool", default_value=0)
        ])


    db.create_table("tex_packages", [
            Column("id", "integer", primary_key=True),
            Column("name", "string")
        ])


    db.create_table("texlive_to_tex", [
            Column("id", "integer", primary_key=True),
            Column("texlive_package_id", "integer", foreign_key=ForeignKey("texlive_packages", "id")),
            Column("tex_package_id", "integer", foreign_key=ForeignKey("tex_packages", "id"))
        ])

    return db

def texpackage_in_database(db: Database, tex_package) -> int | None:

    results = db.SELECT().FROM("tex_packages").WHERE("name", tex_package).run()

    if results:
        return results[0]["id"]
    return None

def cash_tex_package_counts(db: Database):
    
    texlive_packages = db.get_table("texlive_packages")

    for n, texlive_package in enumerate(texlive_packages):
        print(f"Cashing tex packages for package ({n+1}/{len(texlive_packages)}) {texlive_package['name']!r}")
        texlive_package["nr_of_tex_packages"] = blatex.packages.get_number_of_tex_packages(db, texlive_package['name'])
        db.update_entry(texlive_package)

def json_to_database():
    db = create_blank_database()

    with json_file.open() as f:
        scraped_data = json.load(f)

    for n, (texlive_package, tex_packages) in enumerate(scraped_data.items()):
        print(f"{n+1}/{len(scraped_data)} : {texlive_package}")
        
        texlive_package_id = db.add_entry({"name": texlive_package[8:]}, "texlive_packages", fill_null=True)

        for tex_package in tex_packages:
            
            tex_package_id = texpackage_in_database(db, tex_package)

            if not tex_package_id:
                tex_package_id = db.add_entry({"name": tex_package}, "tex_packages")

            db.add_entry({
                "texlive_package_id": texlive_package_id,
                "tex_package_id": tex_package_id
                }, "texlive_to_tex")

    print("Cashing number of tex packages in every texpackage")
    cash_tex_package_counts(db)

    db.close()


if __name__ == "__main__":
    json_to_database()
