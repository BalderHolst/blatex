from sqlite_integrated import Database, Column
import requests

db = Database("src/blatex/resources/packages.db")

col_name = "readme"

if not db.is_column("texlive_packages", col_name):
    db.add_column("texlive_packages", Column(col_name, "text"))


root_url = "https://www.tug.org/docs/latex/"

entries = db.get_table("texlive_packages")
for n, entry in enumerate(entries):
    url = root_url + entry['name'] + "/README"

    print(f"getting readme ({n+1}/{len(entries)}): {url!r}")
    response = requests.get(url)

    if response.status_code == 200:
        print(response.text)
        entry[col_name] = response.text
        db.update_entry(entry)
    else:
        print("ERROR: " + str(response.status_code))

db.close()


