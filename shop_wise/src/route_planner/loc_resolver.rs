use util::{coordinate::Coordinate, store::Store};
use util::search::match_sid_to_brand;
use rusqlite::{Connection, Error, Result};

use crate::database::db_access;

pub fn load_store_locations() -> Result<Vec<Store>> {
    // Get the database connection
    let conn: Connection = db_access::open_database()?;
    let mut stm = conn.prepare("SELECT * FROM store_locations").unwrap();
    let res = stm.query_map( 
        rusqlite::params! {},
        |row: &rusqlite::Row<'_>|->Result<Store, Error>{
            Ok(Store {
                id: row.get(0)?,
                name: row.get(3)?,
                address: row.get(4)?,
                brand: match_sid_to_brand(row.get(1)?).ok_or(Error::InvalidQuery)?,
                location: Coordinate::from_lat_long_f64(row.get(5)?, row.get(6)?),
            })
    })?;
    Ok(res.map(|f: std::prelude::v1::Result<Store, Error>|->Store{f.unwrap()}).collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn test_stores() {
        let res = load_store_locations();
        res.as_ref().expect("Failed to get stores");
        assert!(res.unwrap().len() > 1);
    }
}