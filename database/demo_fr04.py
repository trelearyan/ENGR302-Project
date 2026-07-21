"""
demo_fr04.py — Live demo script for FR-04 (Syon Krishna, ENGR301).

FR-04: "Obtains the price of each item in the shopping list from
        each of the three closest supermarkets."

This script runs against shopwise.db (the SQLite database) and mocks
the upstream components (Shopping List Parser, Item Resolver, Price
Calculator) which aren't built yet.

LIVE DEMO USAGE:
    1. Place this file in the same folder as shopwise.db
    2. Open a terminal in that folder
    3. Run:  python demo_fr04.py
    4. Press Enter to advance between sections (lets you talk over each)
"""
import sqlite3
import os
import sys
import textwrap

DB_PATH = "shopwise.db"


# ────────────────────────────────────────────────────────────────────
# Presentation helpers
# ────────────────────────────────────────────────────────────────────

def banner(title):
    print()
    print("=" * 70)
    print(f"  {title}")
    print("=" * 70)


def section(n, title):
    print()
    print(f"[{n}] {title}")
    print("-" * 70)


def paragraph(text):
    print(textwrap.fill(text, width=70))


def pause(prompt="    (press Enter to continue)"):
    """Pause between sections so you can talk over each."""
    try:
        input(f"\n{prompt}")
    except EOFError:
        pass


# ────────────────────────────────────────────────────────────────────
# Demo sections
# ────────────────────────────────────────────────────────────────────

def show_overview(conn):
    section(1, "Database overview — what's inside shopwise.db")
    paragraph(
        "The database stores price listings for products from each of "
        "the three major NZ supermarket chains. Real scraped data for "
        "Pak'nSave and Woolworths, sample data for New World (scraper "
        "to follow)."
    )
    print()
    print(f"  {'Chain':<14}{'Products':>10}")
    print(f"  {'-'*14:<14}{'-'*10:>10}")
    rows = conn.execute("""
        SELECT s.chain, COUNT(p.id) AS n
        FROM supermarkets s LEFT JOIN products p ON p.supermarket_id = s.id
        GROUP BY s.id ORDER BY s.id
    """).fetchall()
    total = 0
    for chain, n in rows:
        print(f"  {chain:<14}{n:>10,}")
        total += n
    print(f"  {'-'*14:<14}{'-'*10:>10}")
    print(f"  {'Total':<14}{total:>10,}")


def show_single_item_lookup(conn):
    section(2, "FR-04 in action — single-item price lookup")
    paragraph(
        "Scenario: the user has 'butter' on their shopping list. The "
        "Item Resolver maps that to a search term and the Price "
        "Calculator asks the database: what does butter cost at each "
        "supermarket? Here are the three cheapest matches at each chain."
    )
    print()
    print(f"  {'Chain':<12}{'Price':>9}  Product")
    print(f"  {'-'*12:<12}{'-'*9:>9}  {'-'*40}")
    for chain in ("Pak'nSave", "Woolworths", "New World"):
        rows = conn.execute("""
            SELECT s.chain, p.price, p.name, p.volume_size
            FROM products p JOIN supermarkets s ON s.id = p.supermarket_id
            WHERE s.chain = ? AND LOWER(p.name) LIKE '%butter%'
            ORDER BY p.price ASC LIMIT 3
        """, (chain,)).fetchall()
        for ch, price, name, vol in rows:
            label = f"{name} ({vol})" if vol else name
            print(f"  {ch:<12}{'$' + format(price, '.2f'):>9}  {label[:50]}")


def show_shopping_list_comparison(conn):
    section(3, "FR-04 in action — whole shopping list comparison")
    shopping_list = [
        ("milk",          "%milk%"),
        ("bread",         "%bread%"),
        ("eggs",          "%egg%"),
        ("cheese",        "%cheese%"),
        ("peanut butter", "%peanut%butter%"),
    ]
    paragraph(
        "Scenario: the user submits a shopping list. The system asks "
        "the database, for each item, what is the cheapest match at "
        "each chain? The Price Calculator then uses these numbers to "
        "build the Best / Cheapest / Fastest scenarios (FR-06 to FR-08)."
    )
    print()
    header = f"  {'Item':<16}" + "".join(f"{c:>12}" for c in ["Pak'nSave", "Woolworths", "New World"])
    print(header)
    print(f"  {'-'*16:<16}" + "".join(f"{'-'*10:>12}" for _ in range(3)))
    totals = {"Pak'nSave": 0.0, "Woolworths": 0.0, "New World": 0.0}
    for label, pattern in shopping_list:
        row_cells = [f"  {label:<16}"]
        for chain in ("Pak'nSave", "Woolworths", "New World"):
            cheapest = conn.execute("""
                SELECT MIN(p.price)
                FROM products p JOIN supermarkets s ON s.id = p.supermarket_id
                WHERE s.chain = ? AND LOWER(p.name) LIKE ?
            """, (chain, pattern)).fetchone()[0]
            if cheapest is None:
                row_cells.append(f"{'—':>12}")
            else:
                row_cells.append(f"{'$' + format(cheapest, '.2f'):>12}")
                totals[chain] += cheapest
        print("".join(row_cells))
    print(f"  {'-'*16:<16}" + "".join(f"{'-'*10:>12}" for _ in range(3)))
    print(f"  {'TOTAL':<16}" + "".join(f"{'$' + format(totals[c], '.2f'):>12}"
                                       for c in ["Pak'nSave", "Woolworths", "New World"]))
    print()
    cheapest_chain = min(totals, key=totals.get)
    paragraph(
        f"On this list, {cheapest_chain} is cheapest at "
        f"${totals[cheapest_chain]:.2f} before travel cost. The Route "
        f"Planner (Alexander) will add petrol cost on top — that's "
        f"FR-05 — and the Price Calculator (Samuel) combines them for "
        f"FR-06's 'cheapest scenario'."
    )


def show_edge_case(conn):
    section(4, "Edge case — why the Item Resolver is a separate component")
    paragraph(
        "If you naively substring-match the user's word against product "
        "names, you get false matches. Look what happens to 'eggs':"
    )
    print()
    rows = conn.execute("""
        SELECT s.chain, p.name, p.price
        FROM products p JOIN supermarkets s ON s.id = p.supermarket_id
        WHERE LOWER(p.name) LIKE '%eggs%'
        ORDER BY p.price ASC LIMIT 4
    """).fetchall()
    for chain, name, price in rows:
        print(f"  {chain:<12}{'$' + format(price, '.2f'):>9}  {name[:50]}")
    print()
    paragraph(
        "'greggs jelly crystals raspberry flavoured' is not eggs. This "
        "is exactly why our architecture splits the Item Resolver from "
        "the Database. The database's job is to store and serve price "
        "data; the resolver's job is to figure out which products in "
        "the database correspond to a user's typed input."
    )


def show_schema(conn):
    section(5, "Schema — what shipped to the rest of the team")
    paragraph(
        "Two tables. Denormalised on purpose: one row per (chain, "
        "product) listing, indexed for fast lookup by name and chain."
    )
    print()
    for row in conn.execute("""
        SELECT sql FROM sqlite_master
        WHERE type IN ('table','index') AND name NOT LIKE 'sqlite_%'
        ORDER BY type DESC, name
    """):
        print("  " + row[0].replace("\n", "\n  "))
        print()


# ────────────────────────────────────────────────────────────────────
# Main
# ────────────────────────────────────────────────────────────────────

def main():
    if not os.path.exists(DB_PATH):
        print(f"ERROR: cannot find {DB_PATH} in this folder.", file=sys.stderr)
        print("Make sure shopwise.db is in the same folder as this script.", file=sys.stderr)
        sys.exit(1)

    conn = sqlite3.connect(DB_PATH)
    try:
        banner("ShopWise — FR-04 Database Demo  (Syon Krishna)")
        paragraph(
            "FR-04: \"Obtains the price of each item in the shopping list "
            "from each of the three closest supermarkets.\""
        )
        pause()

        show_overview(conn)
        pause()

        show_single_item_lookup(conn)
        pause()

        show_shopping_list_comparison(conn)
        pause()

        show_edge_case(conn)
        pause()

        show_schema(conn)

        print()
        print("=" * 70)
        print("  End of demo — Questions?")
        print("=" * 70)
        print()
    finally:
        conn.close()


if __name__ == "__main__":
    main()
