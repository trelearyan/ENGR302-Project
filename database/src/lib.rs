//! ShopWise database crate.
//!
//! This crate wraps the SQLite database (`shopwise.db`) and exposes
//! a small, typed API for the rest of the project to query. The Price
//! Calculator (Samuel Smith) is the primary consumer; it calls these
//! functions as part of fulfilling FR-04, FR-06, FR-07, and FR-08.

use std::fmt;
use std::str::FromStr;

use bigdecimal::BigDecimal;
use rusqlite::{params_from_iter, Connection, Result};
use util::cost::Cost;
use util::store::{Store, StoreBrand};

/// A unit of measure for a product's package size. Deliberately small —
/// covers what the scrapers actually produce (weight, volume, or a
/// countable pack/each).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Unit {
    Gram,
    Kilogram,
    Millilitre,
    Litre,
    Each,
}

impl fmt::Display for Unit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Unit::Gram => "g",
            Unit::Kilogram => "kg",
            Unit::Millilitre => "mL",
            Unit::Litre => "L",
            Unit::Each => "ea",
        };
        write!(f, "{s}")
    }
}

/// A product's package size, e.g. "500g" -> ProductQuantity { amount: 500.0, unit: Gram }.
/// Replaces the old free-text `volume_size: Option<String>`.
#[derive(Debug, Clone, PartialEq)]
pub struct ProductQuantity {
    pub amount: f64,
    pub unit: Unit,
}

impl fmt::Display for ProductQuantity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", self.amount, self.unit)
    }
}

impl ProductQuantity {
    /// Best-effort parse of the raw size text our scrapers store,
    /// e.g. "500g", "1.5kg", "2L", "6pack", "per kg". Returns `None`
    /// if the text doesn't look like a recognisable quantity.
    #[must_use]
    pub fn parse(raw: &str) -> Option<Self> {
        let raw = raw.trim();
        if raw.is_empty() {
            return None;
        }

        let lower = raw.to_lowercase();
        if lower == "kg" || lower.starts_with("per kg") {
            return Some(Self { amount: 1.0, unit: Unit::Kilogram });
        }

        let digit_end = raw
            .find(|c: char| !c.is_ascii_digit() && c != '.')
            .unwrap_or(raw.len());
        let (num_part, unit_part) = raw.split_at(digit_end);
        let amount: f64 = num_part.parse().ok()?;

        let unit = match unit_part.trim().to_lowercase().as_str() {
            "kg" => Unit::Kilogram,
            "g" => Unit::Gram,
            "l" => Unit::Litre,
            "ml" => Unit::Millilitre,
            "pack" | "pk" | "" => Unit::Each,
            _ => Unit::Each,
        };

        Some(Self { amount, unit })
    }
}

/// One row returned from a product query — the price of a single
/// product at a single supermarket chain.
#[derive(Debug, Clone)]
pub struct ProductPrice {
    pub chain: StoreBrand,
    pub name: String,
    pub price: Cost,
    pub volume_size: Option<ProductQuantity>,
    /// Link to the product photo (None if the chain had none).
    pub image_url: Option<String>,
    /// Loyalty card price, if the product has one (New World Clubcard,
    /// Woolworths Everyday Rewards). `price` is always the shelf price.
    pub member_price: Option<Cost>,
}

/// Which loyalty cards the shopper has switched on. Pak'nSave has no
/// card scheme, so it has no toggle. Default is both off.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct LoyaltyCards {
    /// New World Clubcard (Club+ from June 2026)
    pub clubcard: bool,
    /// Woolworths Everyday Rewards
    pub everyday_rewards: bool,
}

impl ProductPrice {
    /// The price the shopper actually pays: the card price if that
    /// chain's card is on and the card price is lower, else the shelf price.
    #[must_use]
    pub fn price_with(&self, cards: LoyaltyCards) -> Cost {
        let card_on = match self.chain {
            StoreBrand::Newworld => cards.clubcard,
            StoreBrand::Woolworths => cards.everyday_rewards,
            StoreBrand::Paknsave => false,
        };
        match &self.member_price {
            Some(m) if card_on && *m < self.price => m.clone(),
            _ => self.price.clone(),
        }
    }
}

/// Maps a `StoreBrand` to the exact `chain` text stored in the database.
/// Centralised here so there's exactly one place that has to agree with
/// whatever the scraper import tools wrote into the `supermarkets` table.
pub fn chain_name(brand: StoreBrand) -> &'static str {
    match brand {
        StoreBrand::Paknsave => "Pak'nSave",
        StoreBrand::Newworld => "New World",
        StoreBrand::Woolworths => "Woolworths",
    }
}

fn chain_name_to_brand(name: &str) -> Option<StoreBrand> {
    match name {
        "Pak'nSave" => Some(StoreBrand::Paknsave),
        "New World" => Some(StoreBrand::Newworld),
        "Woolworths" => Some(StoreBrand::Woolworths),
        _ => None,
    }
}

/// Converts a raw `(chain, name, price, volume_size, image_url, member_price)` row into a
/// `ProductPrice`. Returns `None` if the chain text doesn't map to a
/// known `StoreBrand` — that row is skipped rather than erroring the
/// whole query out.
fn row_to_product_price(
    chain_text: String,
    name: String,
    price_f64: f64,
    volume_size_text: Option<String>,
    image_url: Option<String>,
    member_price_f64: Option<f64>,
) -> Option<ProductPrice> {
    let chain = chain_name_to_brand(&chain_text)?;

    // Route the f64 through a formatted string rather than a direct
    // float conversion, to avoid binary-float rounding noise landing
    // in a currency type.
    let price = Cost::new(BigDecimal::from_str(&format!("{price_f64:.2}")).ok()?);

    let volume_size = volume_size_text.and_then(|s| ProductQuantity::parse(&s));

    let member_price = member_price_f64
        .and_then(|m| BigDecimal::from_str(&format!("{m:.2}")).ok())
        .map(Cost::new);

    Some(ProductPrice { chain, name, price, volume_size, image_url, member_price })
}

/// Handle to an open SQLite database connection.
pub struct Database {
    conn: Connection,
}

impl Database {
    /// Open the SQLite file at `path`.
    pub fn open(path: &str) -> Result<Self> {
        let conn = Connection::open(path)?;
        Ok(Self { conn })
    }

    /// Return one row per supermarket, with the chain name and how
    /// many product rows it has. Used by the demo binary as a sanity
    /// check.
    pub fn supermarket_summary(&self) -> Result<Vec<(String, i64)>> {
        let mut stmt = self.conn.prepare(
            "SELECT s.chain, COUNT(p.id)
             FROM supermarkets s
             LEFT JOIN products p ON p.supermarket_id = s.id
             GROUP BY s.id
             ORDER BY s.id",
        )?;
        let rows = stmt
            .query_map([], |row| Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?)))?
            .collect::<Result<Vec<_>>>()?;
        Ok(rows)
    }

    /// FR-04's core query: given an item name and the stores the Route
    /// Planner says are in range, return every matching product at
    /// those stores' chains, cheapest first.
    ///
    /// This is the function the Price Calculator will call. For each
    /// shopping list item, it asks: "what's available, where, and for
    /// how much?" — then ranks scenarios from there (FR-06, FR-07, FR-08).
    pub fn find_products(
        &self,
        item_query: &str,
        in_range_stores: &[Store],
    ) -> Result<Vec<ProductPrice>> {
        if in_range_stores.is_empty() {
            return Ok(Vec::new());
        }

        let chain_names: Vec<&str> = in_range_stores
            .iter()
            .map(|store| chain_name(store.brand))
            .collect();

        let placeholders = vec!["?"; chain_names.len()].join(", ");
        let sql = format!(
            "SELECT s.chain, p.name, p.price, p.volume_size, p.image_url, p.member_price
             FROM products p
             JOIN supermarkets s ON s.id = p.supermarket_id
             WHERE s.chain IN ({placeholders})
               AND LOWER(p.name) LIKE LOWER(?)
             ORDER BY p.price ASC
             LIMIT 50"
        );

        let pattern = format!("%{item_query}%");
        let mut params: Vec<&dyn rusqlite::ToSql> = chain_names
            .iter()
            .map(|c| c as &dyn rusqlite::ToSql)
            .collect();
        params.push(&pattern);

        let mut stmt = self.conn.prepare(&sql)?;
        let rows = stmt
            .query_map(params_from_iter(params.iter()), |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, f64>(2)?,
                    row.get::<_, Option<String>>(3)?,
                    row.get::<_, Option<String>>(4)?,
                    row.get::<_, Option<f64>>(5)?,
                ))
            })?
            .collect::<Result<Vec<_>>>()?;

        Ok(rows
            .into_iter()
            .filter_map(|(chain, name, price, vol, img, member)| {
                row_to_product_price(chain, name, price, vol, img, member)
            })
            .collect())
    }

    /// Convenience: for a given item and a set of chains, return the
    /// cheapest matching product across all of them. Returns `None` if
    /// no match exists in any of the given chains, which is how FR-12
    /// (Item Unavailable Notification) gets its signal.
    pub fn cheapest_at(
        &self,
        item_query: &str,
        chains: &[StoreBrand],
    ) -> Result<Option<ProductPrice>> {
        if chains.is_empty() {
            return Ok(None);
        }

        let chain_names: Vec<&str> = chains.iter().map(|b| chain_name(*b)).collect();
        let placeholders = vec!["?"; chain_names.len()].join(", ");
        let sql = format!(
            "SELECT s.chain, p.name, p.price, p.volume_size, p.image_url, p.member_price
             FROM products p
             JOIN supermarkets s ON s.id = p.supermarket_id
             WHERE s.chain IN ({placeholders})
               AND LOWER(p.name) LIKE LOWER(?)
             ORDER BY p.price ASC
             LIMIT 1"
        );

        let pattern = format!("%{item_query}%");
        let mut params: Vec<&dyn rusqlite::ToSql> = chain_names
            .iter()
            .map(|c| c as &dyn rusqlite::ToSql)
            .collect();
        params.push(&pattern);

        let mut stmt = self.conn.prepare(&sql)?;
        let mut rows = stmt.query_map(params_from_iter(params.iter()), |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, f64>(2)?,
                row.get::<_, Option<String>>(3)?,
                row.get::<_, Option<String>>(4)?,
                row.get::<_, Option<f64>>(5)?,
            ))
        })?;

        match rows.next() {
            Some(r) => {
                let (chain, name, price, vol, img, member) = r?;
                Ok(row_to_product_price(chain, name, price, vol, img, member))
            }
            None => Ok(None),
        }
    }

    /// Same as `find_products`, but ranked by what the shopper pays with
    /// the given cards switched on (card price where it applies).
    /// `find_products` itself is unchanged and still ranks by shelf price.
    pub fn find_products_with_cards(
        &self,
        item_query: &str,
        in_range_stores: &[Store],
        cards: LoyaltyCards,
    ) -> Result<Vec<ProductPrice>> {
        let mut products = self.find_products(item_query, in_range_stores)?;
        products.sort_by(|a, b| a.price_with(cards).cmp(&b.price_with(cards)));
        Ok(products)
    }

    /// Same as `cheapest_at`, but the cheapest after card prices are applied.
    pub fn cheapest_at_with_cards(
        &self,
        item_query: &str,
        chains: &[StoreBrand],
        cards: LoyaltyCards,
    ) -> Result<Option<ProductPrice>> {
        let stores: Vec<Store> = chains
            .iter()
            .map(|b| Store { brand: *b, location: util::coordinate::Coordinate::wellington() })
            .collect();
        Ok(self
            .find_products_with_cards(item_query, &stores, cards)?
            .into_iter()
            .next())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_DB: &str = "../data/shopwise.db";

    #[test]
    fn opens_database() {
        let db = Database::open(TEST_DB).expect("database file should be readable");
        let summary = db.supermarket_summary().expect("query should run");
        assert!(!summary.is_empty(), "expected at least one supermarket chain");
    }

    fn product(chain: StoreBrand, shelf: &str, member: Option<&str>) -> ProductPrice {
        ProductPrice {
            chain,
            name: "x".to_owned(),
            price: Cost::new(BigDecimal::from_str(shelf).unwrap()),
            volume_size: None,
            image_url: None,
            member_price: member.map(|m| Cost::new(BigDecimal::from_str(m).unwrap())),
        }
    }

    #[test]
    fn card_price_only_applies_when_that_card_is_on() {
        let nw = product(StoreBrand::Newworld, "5.00", Some("4.00"));
        let off = LoyaltyCards::default();
        let on = LoyaltyCards { clubcard: true, everyday_rewards: false };
        assert_eq!(nw.price_with(off), nw.price);
        assert_eq!(nw.price_with(on), *nw.member_price.as_ref().unwrap());

        // Wrong card for the chain: no discount
        let wool = product(StoreBrand::Woolworths, "5.00", Some("4.00"));
        assert_eq!(wool.price_with(on), wool.price);

        // Pak'nSave never has a card price
        let pak = product(StoreBrand::Paknsave, "5.00", Some("1.00"));
        let both = LoyaltyCards { clubcard: true, everyday_rewards: true };
        assert_eq!(pak.price_with(both), pak.price);
    }
}