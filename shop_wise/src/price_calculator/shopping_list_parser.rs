use std::ops::Add;
use util::search::ShoppingItemQuery;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ListParserError {
    // use these u32's to return the exact place where invalidness was
    // found, this way we can present an error in the GUI if we want
    InvalidCsv(u32),
    IllegalCharacter(u32),
    ImproperLinebreak(u32),
}

struct CSV {
    field_len: i32,
    fields: Vec<String>,
}

/// Parse a csv into a list of queries
/// <br>
/// Returns:
/// <br>
/// Ok(&[&str]) if the list of queries could be produced
/// <br>
/// Err(ListParserError::InvalidCsv(column) if the parameter is invalid CSV.
/// <br>
/// Err(ListParserError::IllegalCharacter(column) if the parameter contains
/// an illegal character.
/// <br>
/// Err(ImproperLinebreak::IllegalCharacter(column) if the parameter contains
/// an invalid linebreak sequence
fn parse(csv_of_shopping_list_items: &str) -> Result<Vec<ShoppingItemQuery>, ListParserError> {
    let parsed: Result<CSV, ListParserError> = parse_csv(csv_of_shopping_list_items);
    if (parsed.is_err()) {
        return Err(parsed.err().unwrap());
    }
    todo!();
}

/// Parse a csv into a vector of strings.
/// The Strings will be copied, and no references are made to the original csv str
/// <br>
/// Returns:
/// <br>
/// Ok(&[&str]) if the string could be parsed as valid CSV and split into
/// an array of sanitised strings
/// <br>
/// Err(ListParserError::InvalidCsv(column) if the parameter is invalid CSV.
/// <br>
/// Err(ListParserError::IllegalCharacter(column) if the parameter contains
/// an illegal character.
/// <br>
/// Err(ImproperLinebreak::IllegalCharacter(column) if the parameter contains
/// an invalid linebreak sequence
fn parse_csv(csv: &str) -> Result<CSV, ListParserError> {
    // begin with empty field length
    let mut field_len: i32 = 0;
    // Start working string with a reasonable capacity to avoid constant re-allocation
    let mut fields: Vec<String> = Vec::new();
    let mut working: String = String::with_capacity(100);
    // 0 is ready to parse a new field
    // 1 is parsing a field surrounded by quotes
    // 2 is parsing a field surrounded by quotes, with one pending quote
    // 3 is parsing a field without quoutes
    let mut quote_phase: i32 = 0; 
    let mut return_char: bool = false;
    let mut err_counter: u32 = 0;
    'whole: for c in csv.chars() {
        if (quote_phase != 1 && c != '\n' && return_char) {
            // No \n after \r - invalid
            return Err(ListParserError::ImproperLinebreak(err_counter));
        }
        match c {
            '"' => {
                match quote_phase {
                    0 => {
                        // Start parsing the field
                        quote_phase = 1;
                    }
                    1 => {
                        // Read one quote
                        quote_phase = 2;
                    }
                    2 => {
                        // Read second quote and so add one quote to field
                        working.push_str("\"");
                        quote_phase = 1;
                    }
                    _ => {
                        // Cannot have quotes if not within quotes - invalid
                        return Err(ListParserError::IllegalCharacter(err_counter));
                    }
                }
            }
            ',' => {
                match quote_phase {
                    1 => {
                        // In quotes, so comma just character to add to field
                        working.push(c);
                    }
                    _ => {
                        // Close
                        fields.push(working.clone());
                        working.clear();
                    }
                }
            }
            '\r' => {
                match quote_phase {
                    1 => {
                        // In quotes, so \r just character to add to field
                        working.push(c);
                    }
                    _ => {
                        // Mark return character
                        return_char = true;
                    }
                }
            }
            '\n' => {
                match quote_phase {
                    1 => {
                        // In quotes, so \n just character to add to field
                        working.push(c);
                    }
                    _ => {
                        // Mark newline character
                        if (return_char) {
                            // If newline reached
                            if (working.len() == 0) {
                                // End of file reached as empty line
                                if field_len == 0 {
                                    // If field length not yet set whole file empty - invalid
                                    return Err(ListParserError::InvalidCsv(err_counter));
                                }
                                break 'whole;
                            } else {
                                // Add record to file
                                fields.push(working.clone());
                                working.clear();
                                if field_len == 0 {
                                    // If field length not yet set, set it
                                    field_len = working.len() as i32;
                                } else if (fields.len() as i32) % field_len != 0 {
                                    // Records do not all have the same number of fields - invalid
                                    return Err(ListParserError::InvalidCsv(err_counter));
                                }
                            }
                            return_char = false;
                        } else {
                            // No \r before \n - invalid
                            return Err(ListParserError::ImproperLinebreak(err_counter));
                        }
                    }
                }
            }
            _ => {
                match quote_phase {
                    0 => {
                        // No starting quote
                        quote_phase = 3;
                        working.push(c);
                    }
                    2 => {
                        // No second quote - invalid
                        return Err(ListParserError::InvalidCsv(err_counter));
                    }
                    _ => {
                        working.push(c);
                    }
                }
            }
        }
        err_counter += 1;
    }

    if (fields.len() as i32) % field_len != 0 {
        // Records do not all have the same number of fields
        return Err(ListParserError::InvalidCsv(err_counter));
    }

    // Return CSV
    Ok(CSV {
        field_len,
        fields: fields,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
}
