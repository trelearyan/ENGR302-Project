"""

Code borrowed from Syon Krishna

products
id, supermarket_id, name, price, volume_size, image_url
int, int, text, real, text, text

supermarket
id, chain
int, text

conn.execute("select * from sqlite_master").fetchall()
[('table', 'supermarkets', 'supermarkets', 2,
'CREATE TABLE supermarkets
(id INTEGER PRIMARY KEY AUTOINCREMENT,chain TEXT NOT NULL UNIQUE)'),

('index', 'sqlite_autoindex_supermarkets_1', 'supermarkets', 3, None),

('table', 'sqlite_sequence', 'sqlite_sequence', 4,
'CREATE TABLE sqlite_sequence(name,seq)'),

('table', 'products', 'products', 5,
'CREATE TABLE products (
id INTEGER PRIMARY KEY AUTOINCREMENT,
supermarket_id INTEGER NOT NULL,
name           TEXT    NOT NULL,
price          REAL    NOT NULL,
volume_size    TEXT,
image_url TEXT,
FOREIGN KEY (supermarket_id)
REFERENCES supermarkets(id)\n        )'),

('index', 'idx_products_name', 'products', 6,
'CREATE INDEX idx_products_name        ON products(name)'),

('index', 'idx_products_supermarket', 'products', 7,
'CREATE INDEX idx_products_supermarket ON products(supermarket_id)')]


e.g.
SELECT s.chain, p.name, p.price
FROM products p JOIN supermarkets s ON s.id = p.supermarket_id
WHERE p.name LIKE '%Eggs%'
ORDER BY p.price ASC LIMIT 4

"""

import sys, sqlite3, os

DB_PATH = "shop_wise/src/demo_db/shopwise.db"
OUT_PATH = "shop_wise/src/demo_db/out.txt"

def main():
    if not os.path.exists(DB_PATH):
        print(f"ERROR: cannot find {DB_PATH} in this folder.", file=sys.stderr)
        print("Make sure shopwise.db is in the same folder as this script.", file=sys.stderr)
        sys.exit(1)
    # Parse args
    args = sys.argv[1:]
    if len(args) != 2 or args[0] != "--query":
        sys.exit("argument(s) not valid")
    query = args[1]
    conn = sqlite3.connect(DB_PATH)
    try:
        rows = conn.execute(args[1]).fetchall()
        #Write in csv format
        out = "\r\n".join(
            [",".join(["\""+str(cell).replace("\"", "\"\"")+"\""
                       for cell in record])
                for record in rows])+"\r\n"
        # Write ASCII encoding to file
        # due to Rose spelling at Pak'n'Save
        with open(OUT_PATH, "wb") as f:
          f.write(out.encode(encoding="ascii",errors="replace"))
        print("Write successful")
    except:
        sys.exit("Error in executing sql statement")
    finally:
        conn.close()

if __name__ == "__main__":
    main()
