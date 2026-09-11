use std::fmt;

use util::search::{match_to_unit, ShoppingItemQuery};

#[derive(Debug)]
pub enum CsvError {
    Malformed(String),
    Row { line: usize, reason: String },
    NoItems,
}
impl fmt::Display for CsvError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CsvError::Malformed(detail) => write!(f, "CSV is not valid:  {detail}"),
            CsvError::Row { line, reason } => write!(f, "has error in line {line}: {reason}"),
            CsvError::NoItems => write!(f, "file  is empty"),
        }
    }
}

impl std::error::Error for CsvError {}


pub fn read_csv(text: &str) -> Result<Vec<ShoppingItemQuery>, CsvError> {
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(true)
        .trim(csv::Trim::All)
        .flexible(true)
        .from_reader(text.as_bytes());

    let mut items = Vec::new();

    for (index, record) in reader.records().enumerate() {
        let line = index + 2;
        let record = record.map_err(|e| CsvError::Malformed(e.to_string()))?;

        // blank lines -> skip
        if record.iter().all(str::is_empty) {
            continue;
        }

        let name = record.get(0).unwrap_or("").to_owned();
        if name.is_empty() {
            return Err(CsvError::Row {
                line,
                reason: "item name is missing".to_owned(),
            });
        }

        let quantity_text = record.get(1).unwrap_or("");

        //TODO: won't read decimal e.g 2.5 kg. find a way to fix
        let quantity: u32 = quantity_text.parse().map_err(|_| CsvError::Row {
            line,
            reason: format!("\"{quantity_text}\" is decimal"),
        })?;
        if quantity == 0 {
            return Err(CsvError::Row {
                line,
                reason: "check your quantity".to_owned(),
            });
        }

        let unit_text = record.get(2).unwrap_or("");
        if match_to_unit(unit_text).is_none() {
            return Err(CsvError::Row {
                line,
                reason: format!("\"{unit_text}\" is not recognised"),
            });
        }

        items.push(ShoppingItemQuery {
            name,
            quantity,
            unit: unit_text.to_owned(),
        });
    }

    if items.is_empty() {
        return Err(CsvError::NoItems);
    }

    Ok(items)
}

pub fn write_to_csv<'a>(items: impl IntoIterator<Item = &'a ShoppingItemQuery>) -> String  {
    let mut writer = csv::Writer::from_writer(Vec::new());

    let _ = writer.write_record(["name", "quantity", "unit"]);
    for item in items {
        let _ = writer.write_record([
            item.name.as_str(),
            &item.quantity.to_string(),
            item.unit.as_str(),
        ]);
    }

    writer
        .into_inner()
        .ok()
        .and_then(|bytes| String::from_utf8(bytes).ok())
        .unwrap_or_default()
}

