from setuptools import setup

with open("README.md", "r") as f:
    description = f.read()

setup(
    name = "blatex",
    version = '0.0.3',
    author = "Balder Holst",
    author_email = "balderwh@gmail.com",
    packages = ["blatex"],
    description = "Simple cli tool for managing latex projects.",
    long_description = description,
    long_description_content_type = "text/markdown",
    package_dir={'': 'src'},
    package_data={"blatex": [
        "resources/templates/*",
        "resources/default-local-config.json",
        "resources/default-global-config.json"
        ]},
    install_requires = [
        "termcolor",
        "click",
        "pathlib"
        ],
    entry_points = {
        'console_scripts': ['blatex=blatex:blatex']
        },
    python_requires = ">=3.7",
    url = "https://github.com/BalderHolst/blatex",
    license = "MIT",
)
