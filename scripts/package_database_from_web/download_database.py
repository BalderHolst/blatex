import requests
from bs4 import BeautifulSoup

import re

import os
from pathlib import Path

from sqlite_integrated import Database, Column, ForeignKey

db_file = "scripts/package_database_from_web/packages.db"

def get_blank_db() -> Database:
    if Path(db_file).exists():
        print(f"removing {db_file!r}")
        os.remove(db_file)

    db = Database(db_file, new=True)

    db.create_table("maintainers", [
            Column("id", "integer", primary_key=True),
            Column("name", "text", not_null=True)
        ])

    db.create_table("texlive_packages", [
            Column("id", "integer", primary_key=True),
            Column("name", "string"),
            Column("contrib", "bool", not_null=True)
        ])

    db.create_table("miktex_packages", [
            Column("id", "integer", primary_key=True),
            Column("name", "string")
        ])

    db.create_table("tex_to_texlive", [
            Column("id", "integer", primary_key=True),
            Column("tex_package_id", "int", foreign_key=ForeignKey("tex_packages", "id")),
            Column("texlive_package_id", "int", foreign_key=ForeignKey("texlive_packages", "id"))
        ])

    db.create_table("tex_to_miktex", [
            Column("id", "integer", primary_key=True),
            Column("tex_package_id", "int", foreign_key=ForeignKey("tex_packages", "id")),
            Column("miktex_package_id", "int", foreign_key=ForeignKey("miktex_packages", "id"))
        ])


    db.create_table("tex_packages", [
            Column("id", "integer", primary_key=True),
            Column("name", "string"),
            Column("readme", "text"),
            Column("ctan_url", "string"),
        ])

    db.create_table("tex_package_maintainers", [
            Column("id", "integer", primary_key=True),
            Column("tex_package_id", "int", foreign_key=ForeignKey("tex_packages", "id")),
            Column("maintainer_id", "int", foreign_key=ForeignKey("maintainers", "id")),
        ])
        


    db.create_table("main_packages", [
            Column("id", "int", primary_key=True),
            Column("texlive_package_id", "int", foreign_key=ForeignKey("texlive_packages", "id")),
            Column("miktex_package_id", "int", foreign_key=ForeignKey("miktex_packages", "id"))
        ])

    db.create_table("tex_documentation", [
            Column("id", "int", primary_key=True),
            Column("pdf_url", "string"),
            Column("tex_package_id", "int", foreign_key=ForeignKey("tex_packages", "id"))
        ])

    return(db)

def get_package_links():
    base_url = "https://www.ctan.org"

    alphabet = "ABCDEFGHIJKLMNOPQRSTUVWXYZ"

    package_links = []

    for letter in alphabet:
        url = base_url + "/pkg/:" + letter

        print(f"getting links from: {url!r}")

        response = requests.get(url)

        if response.status_code != 200:
            print(f"Could not access {letter!r}")
            continue

        soup = BeautifulSoup(response.content, "html.parser")

        links = soup.find_all(class_ = "dt")

        for link in links:
            package_links.append(base_url + link.a['href'])

    print(f"Found links to {len(package_links)} packages!")
    return(package_links)

def get_website(url):
    for _ in range(5):
        response = requests.get(url)
        if response.status_code == 200:
            break
        print(f"ERROR: {response.status_code}... retrying.")

    if response.status_code != 200:
        return None
    return response.content

def get_website_soup(url):

    content = get_website(url)

    if not content:
        return None

    soup = BeautifulSoup(content, "html.parser")
    return soup

def get_ctan_readme(soup):
        readme_urls = [tag['href'] for tag in soup.find_all(class_ = "doc-readme")]
        readmes = []
        for n, url in enumerate(readme_urls):
            print(f"\tgetting readme ({n+1}/{len(readme_urls)}): {url!r}...")
            readme = get_website(url)
            if readme:
                readmes.append(str(readme))

        return "\n\n\n".join(readmes)

def get_ctan_pdf_documentation(soup):
    urls = [tag['href'] for tag in soup.find_all(class_ = "doc-filetype.pdf")]

    for url in urls:
        print(f"\tfound documentation: {url!r}")

    return urls

def get_ctan_maintainers(soup):
    table = soup.find(class_ = "entry")

    maintainers = []

    for entry in table.children:
        if entry.name == "tr":
            gen = entry.children
            title = next(gen).text

            if title == "Maintainer":
                data = list(next(gen).children)
                for tag in data:
                    if(tag.name == "a"):
                        maintainers.append(tag.text)

    return(maintainers)

def get_ctan_contained_in(soup):
    table = soup.find(class_ = "entry")

    raw_packages = []

    for entry in table.children:
        if entry.name == "tr":
            gen = entry.children
            title = next(gen).text

            if re.search("Contained.in", title):
                data = list(next(gen).children)
                for tag in data:
                    raw_packages.append(tag.text)

    packages = {}
    name = "no-name"

    for rp in raw_packages:
        if rp == "":
            continue
        if re.search(" as ", rp):
            packages[name] = rp.replace(" as ", "")
            continue
        name = rp.replace("TeX\u2009Live", "texlive").replace("MiKTeX", "miktex")

        

    return packages


        
def add_package_to_database(db: Database, name, readme, ctan_url, doc_urls, maintainers, contained_in: dict):

    # Add tex package
    tex_package_id = db.add_entry(
            {
                "name": name,
                "readme": readme,
                "ctan_url": ctan_url
                }
            , "tex_packages"
            )

    # Add maintainers
    for maintainer_name in maintainers:
        res = db.SELECT().FROM("maintainers").WHERE("name", maintainer_name).run()
        if res:
            maintainer_id = res[0]["id"]
        else:
            maintainer_id = db.add_entry({"name": maintainer_name}, "maintainers")

        db.add_entry({
                "tex_package_id": tex_package_id, 
                "maintainer_id": maintainer_id
            }, 
            "tex_package_maintainers"
        )

    # add tex documentation
    for doc_url in doc_urls:
        db.add_entry(
            {
                "pdf_url": doc_url,
                "tex_package_id": tex_package_id
                },
            "tex_documentation"
        )

    # Add texlive and miktex
    for ptype, package_name in contained_in.items():
        texlive_package_id = None
        miktex_package_id = None

        if ptype == "texlive" or ptype == "texlive Contrib":

            contrib = ptype == "texlive Contrib"

            texlive_package_id = db.add_entry({
                "name": package_name,
                "contrib": contrib
                }, "texlive_packages")

            db.add_entry({
                "tex_package_id": tex_package_id,
                "texlive_package_id": texlive_package_id
                }, "tex_to_texlive")

        elif ptype == "miktex":
            miktex_package_id = db.add_entry({
                "name": package_name
                }, "miktex_packages")

            db.add_entry({
                "tex_package_id": tex_package_id,
                "miktex_package_id": miktex_package_id
                }, "tex_to_miktex")

        else:
            raise Exception(f"Dont know what to to when package is contained in {ptype!r}.")

    db.save()

def download_database():
    db = get_blank_db()
    links = get_package_links()

    for n, link in enumerate(links):

        print(f"Scraping link ({n+1}/{len(links)}): {link!r}")

        soup = get_website_soup(link)
        if not soup:
            continue

        name = Path(link).stem
        readme = get_ctan_readme(soup)
        pdf_documentation_urls = get_ctan_pdf_documentation(soup)
        maintainers = get_ctan_maintainers(soup)
        contained_in = get_ctan_contained_in(soup)

        add_package_to_database(db, name, readme, link, pdf_documentation_urls, maintainers, contained_in)

        print("\n")


if __name__ == "__main__":
    download_database()
