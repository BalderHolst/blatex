from pathlib import Path
import subprocess
import json

install_location = Path("/usr/share/texlive/texmf-dist/tex/latex")
dnf_packages_list = Path("./find_package_names/dnf_packages.txt")
json_out_file = Path("find_package_names/packages.json")

def find_sty(directory: Path):
    if not directory.exists():
        return []

    stys = []
    for file in directory.iterdir():
        if file.is_file():
            if file.suffix == ".sty":
                stys.append(file)
        elif file.is_dir():
            stys.extend(find_sty(file))
    return(stys)

def register_packages(dnf_package_name: str):
    stys = find_sty(install_location)
    tex_package_names = [path.stem for path in stys]

    with json_out_file.open("r") as f:
        l = json.load(f)

    l[dnf_package_name] = tex_package_names

    with json_out_file.open("w") as f:
        json.dump(l, f, indent=4)

def is_registered(dnf_package_name: str):
    with json_out_file.open() as f:
        l = json.load(f)

    return (dnf_package_name in l)




with dnf_packages_list.open() as f:
    dnf_packages = [line[:-1] for line in f.readlines()] # remove newline charecters

# dnf_packages = dnf_packages[100:103] # TODO REMOVE!

for n, dnf_package in enumerate(dnf_packages):

    if is_registered(dnf_package):
        print(f"Skipping {dnf_package!r} as it is already registered")
        continue
    
    print(f"Registering {dnf_package} ({n+1}/{len(dnf_packages)}).... ", end="")

    subprocess.run(f"sudo dnf install {dnf_package} -y", shell=True, stdout=subprocess.DEVNULL)

    register_packages(dnf_package)

    subprocess.run(f"sudo dnf remove {dnf_package} -y", shell=True, stdout=subprocess.DEVNULL)
    
    print("done.")


print("Done!")
