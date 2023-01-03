import zipfile
from pathlib import Path

templatedir = Path("templates")

def choose_template():
    templates = [[f.stem, f] for f in templatedir.iterdir()]

    print("Choose a template:")
    for n, template in enumerate(templates):
        print("\t" + str(n) + ": " + str(template[0]))

    while True:
        nr = input("Template index: ")
        try:
            nr = int(nr)
        except ValueError:
            print(f"{nr!r} is not a valus template index.")
            continue
        if nr < len(templates) and nr >= 0:
            break
        print(f"You must input a number between 0 and {len(templates) - 1}")

    return(templates[nr][1])

        
def copy_template(templatefile: Path | str, destination: Path | str):
    with zipfile.ZipFile(templatefile, mode="r") as archive:
         archive.extractall(destination)

if __name__ == "__main__":
    template = choose_template()
    copy_template(template, "test")
