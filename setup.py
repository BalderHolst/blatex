from setuptools import setup

with open("README.md", "r") as f:
    description = f.read()

setup(
    name = "blatex",
    version = '0.0.0',
    author = "Balder Holst",
    author_email = "balderwh@gmail.com",
    packages = ["blatex"],
    description = "Cli tool for managing latex projects.",
    long_description = description,
    long_description_content_type = "text/markdown",
    package_dir={'': 'src'},
    package_data={"blatex": ["templates/*"]},
    python_requires = ">=3.7",
    install_requires = [
        "click",
        "zipfile",
        "pathlib"
        ],
    entry_points = {
        'console_scripts': ['blatex=blatex:blatex']
        }
)
