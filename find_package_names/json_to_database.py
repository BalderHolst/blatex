import json
from pathlib import Path
import os
from sqlite_integrated import Database, Column, ForeignKey

db_file = Path("dnf.db")

json_file = Path("./packages.json")

if db_file.exists():
    os.remove(db_file)

db = Database(db_file, new=True)

db.create_table("dnf_packages", [
        Column("id", "integer", primary_key=True),
        Column("name", "string")
    ])


db.create_table("tex_packages", [
        Column("id", "integer", primary_key=True),
        Column("name", "string")
    ])


db.create_table("dnf_to_tex", [
        Column("id", "integer", primary_key=True),
        Column("dnf_package_id", "integer", foreign_key=ForeignKey("dnf_packages", "id")),
        Column("tex_package_id", "integer", foreign_key=ForeignKey("tex_packages", "id"))
    ])

# db.overview()



with json_file.open() as f:
    scraped_data = json.load(f)

for n, (dnf_package, tex_packages) in enumerate(scraped_data.items()):
    print(f"{dnf_package} : {n+1}/{len(scraped_data)}")

    db.add_entry({"name": dnf_package}, "dnf_packages")

    dnf_package_id = n + 1

    for i, tex_package in enumerate(tex_packages):
        print(f"{dnf_package} : {n+1}/{len(scraped_data)} - {i+1}/{len(tex_packages)}")
        
        # TODO only do this if the package is not added
        db.add_entry({"name": tex_package}, "tex_packages")
        tex_package_id = len(db.get_table("tex_packages"))

        db.add_entry({
            "dnf_package_id": dnf_package_id, 
            "tex_package_id": tex_package_id
            }, "dnf_to_tex")



db.save()

