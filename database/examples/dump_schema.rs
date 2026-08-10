use rusqlite::Connection;

fn main() {
    let conn = Connection::open("../data/shopwise.db")
        .expect("could not open database — run this from inside the database/ folder");

    let mut stmt = conn
        .prepare("SELECT sql FROM sqlite_master WHERE type = 'table'")
        .unwrap();

    let schemas = stmt
        .query_map([], |row| row.get::<_, String>(0))
        .unwrap();

    for schema in schemas {
        println!("{}\n", schema.unwrap());
    }
}