from sqlite_integrated import Database, Column
import pkg_resources

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

    return db.cursor.fetchall()


def find_texlive_packages(db: Database, tex_package: str):

    sql = """SELECT tl.* FROM tex_packages t
    JOIN texlive_to_tex ttt ON ttt.tex_package_id = t.id
    JOIN texlive_packages tl ON tl.id = ttt.texlive_package_id
    WHERE t.name = """ + f"'{tex_package}'"

    db.cursor.execute(sql)

    return db.cursor.fetchall()

# TODO make common packages appear first
def get_texlive_recommendations(tex_package, count=5):
    db = get_db()

    texlive_packages = find_texlive_packages(db, tex_package)

    recommended_texlive_packages = []

    for _ in range(count):
        best = None
        for texlive_package in texlive_packages:
            if texlive_package in recommended_texlive_packages:
                continue
            if not best or texlive_package[2] < best[2]:
                best = texlive_package
        recommended_texlive_packages.append(best)



    db.conn.close()

    return [p[1] for p in recommended_texlive_packages]


def update_cashed_tex_package_counts():
    db = get_db()
    
    cashe_col_name = "nr_of_tex_packages" 
    if not "nr_of_tex_packages" in [col.name for col in db.get_table_cols("texlive_packages")]:
        db.add_column("texlive_packages", Column("nr_of_tex_packages", "int"))

    for texlive_package in db.get_table("texlive_packages"):
        print(f"Cashing for package {texlive_package['name']!r}...")
        texlive_package[cashe_col_name] = len(find_tex_packages(db, texlive_package['name']))
        db.update_entry(texlive_package)

    db.close()
    print("DONE!")


if __name__ == "__main__":
    for p in get_texlive_recommendations("graphicx", 10):
        print(p)
