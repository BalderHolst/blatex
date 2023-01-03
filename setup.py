from setuptools import setup, find_packages

setup(
    name = "blatex",
    version = '0.0.0',
    author = "Balder Holst",
    author_email = "balderwh@gmail.com",
    packages = find_packages(),
    package_dir={'': 'src'},
    install_requires = [
        "click"
        ],
    entry_points = """
    [console_scripts]
    blatex=blatex:blatex
    """
)
