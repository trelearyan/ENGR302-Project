use std::ops::Add;
use util::search::{ShoppingItemQuery, match_to_unit};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ListParserError {
    // use these u32's to return the exact place where invalidness was
    // found, this way we can present an error in the GUI if we want
    CSVInvalid(u32),
    CSVIllegalCharacter(u32),
    CSVImproperLinebreak(u32),
    // use this u32 to indicate the invalid parameter
    ParseInvalidShape(u32),
    // use the String to indicate reason and the u32 for line number
    LineNotReadable(String, u32),
}

#[derive(Debug, Eq, PartialEq)]
pub struct CSV {
    pub field_len: u32,
    pub fields: Vec<String>,
}

/// Parse a csv into a list of queries
/// <br>
/// Returns:
/// <br>
/// Ok(&[&str]) if the list of queries could be produced
/// <br>
/// Err(ListParserError::CSVInvalid(column) if the parameter is invalid CSV.
/// <br>
/// Err(ListParserError::CSVIllegalCharacter(column) if the parameter contains
/// an illegal character.
/// <br>
/// Err(ListParserError::CSVImproperLinebreak(column) if the parameter contains
/// an invalid linebreak sequence
/// <br>
/// Err(ListParserError::ParseInvalidShape(width) if the csv is a different record
/// size than the intended 3
/// <br>
/// Err(ListParserError::LineNotReadable(Reason, line) if a line cannot be parsed
/// into a shopping query
pub fn parse(csv_of_shopping_list_items: &str) -> Result<Vec<ShoppingItemQuery>, ListParserError> {
    // Parse input and validate parsing
    let parsed: CSV;
    {
        let parse_result: Result<CSV, ListParserError> = parse_csv(csv_of_shopping_list_items);
        if (parse_result.is_err()) {
            return Err(parse_result.err().unwrap());
        }
        parsed = parse_result.unwrap();
    }
    // Check that has the expected field size
    if (parsed.field_len != 3) {
        return Err(ListParserError::ParseInvalidShape(parsed.field_len));
    }
    // Check that their are enough fields - SHOULDN'T EVER FAIL GIVEN ABOVE
    if (parsed.fields.len() < 3) {
            return Err(ListParserError::LineNotReadable("Not enough fields".to_owned(), 0));
    }
    // Second field should be a string representation of an integer, unless header
    let mut iline: usize = 0;
    if (parsed.fields.get(1).unwrap().parse::<u32>().is_err()) {
        // Assume header and so try 5th
        if (parsed.fields.len() < 6 || parsed.fields.get(4).unwrap().parse::<u32>().is_err()) {
            return Err(ListParserError::LineNotReadable("More than one header or malformed".to_owned(), 1));
        }
        iline = 1;
    }
    let mut shopping_query: Vec<ShoppingItemQuery> = Vec::new();
    while (iline < parsed.fields.len() / 3) {
        let name: String = parsed.fields.get(iline*3).unwrap().to_string();
        if (name.is_empty()) {
            return Err(ListParserError::LineNotReadable("Name is empty".to_owned(), iline as u32));
        }
        let quantity: Result<u32, std::num::ParseIntError>= parsed.fields.get(iline*3+1).unwrap().parse::<u32>();
        if (quantity.is_err()) {
            return Err(ListParserError::LineNotReadable("Could not read quantity".to_owned(), iline as u32));
        }
        let unit: String = parsed.fields.get(iline*3+2).unwrap().to_string();
        if (match_to_unit(unit.as_ref()).is_none()) {
            return Err(ListParserError::LineNotReadable("Unit not valid".to_owned(), iline as u32));
        }
        shopping_query.push(ShoppingItemQuery {
            name: name,
            quantity: quantity.unwrap(),
            unit: unit
        });
        iline += 1;
    }
    Ok(shopping_query)
}

/// Parse a string representation of a csv into same-size records represented by CSV
/// The Strings will be copied, and no references are made to the original csv str
/// (simplification made from Cow as some strings are sanitised, but short-lived so
/// not neccessary to hold reference to original csv string -> ideally drop csv string
/// immediately after parsing)
/// <br>
/// Returns:
/// <br>
/// Ok(&[&str]) if the string could be parsed as valid CSV and split into
/// an array of sanitised strings
/// <br>
/// Err(ListParserError::CSVInvalid(column) if the parameter is invalid CSV.
/// <br>
/// Err(ListParserError::CSVIllegalCharacter(column) if the parameter contains
/// an illegal character.
/// <br>
/// Err(ListParserError::CSVImproperLinebreak(column) if the parameter contains
/// an invalid linebreak sequence
pub fn parse_csv(csv: &str) -> Result<CSV, ListParserError> {
    // begin with empty field length
    let mut field_len: u32 = 0;
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
            return Err(ListParserError::CSVImproperLinebreak(err_counter));
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
                        return Err(ListParserError::CSVIllegalCharacter(err_counter));
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
                        quote_phase = 0;
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
                                    return Err(ListParserError::CSVInvalid(err_counter));
                                }
                                break 'whole;
                            } else {
                                // Add record to file
                                fields.push(working.clone());
                                working.clear();
                                quote_phase = 0;
                                if field_len == 0 {
                                    // If field length not yet set, set it
                                    field_len = fields.len() as u32;
                                } else if (fields.len() as u32) % field_len != 0 {
                                    // Records do not all have the same number of fields - invalid
                                    return Err(ListParserError::CSVInvalid(err_counter));
                                }
                            }
                            return_char = false;
                        } else {
                            // No \r before \n - invalid
                            return Err(ListParserError::CSVImproperLinebreak(err_counter));
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
                        return Err(ListParserError::CSVInvalid(err_counter));
                    }
                    _ => {
                        working.push(c);
                    }
                }
            }
        }
        err_counter += 1;
    }
    if (!working.is_empty()) {
        fields.push(working.clone());
        working.clear();
        if field_len == 0 {
            // If field length not yet set, set it
            field_len = fields.len() as u32;
        }
    }

    if (field_len == 0) {
        // Field len has not been set
        panic!();
    }

    if (fields.len() as u32) % field_len != 0 {
        // Records do not all have the same number of fields
        return Err(ListParserError::CSVInvalid(err_counter));
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

    // Convert string literals into owned strings for testing
    macro_rules! string_vec {
        ($($s:expr),* $(,)?) => {
            vec![$($s.to_owned()),*]
        };
    }

    #[test]
    fn test_csvparse_basic() {
        let csvtext: &str = "id,name,email\r\n1,John,john.doe@example.com\
                            \r\n2,Jane,janey72@test.org";
        assert_eq!(Ok(CSV {
                field_len: 3,
                fields: string_vec![
                    "id", "name", "email",
                    "1", "John", "john.doe@example.com",
                    "2", "Jane", "janey72@test.org"
                ],
            }),
            parse_csv(csvtext)
        );
    }

    #[test]
    fn test_csvparse_oneliner() {
        let csvtext: &str = "id,name,email";
        assert_eq!(Ok(CSV {
                field_len: 3,
                fields: string_vec![
                    "id", "name", "email",
                ],
            }),
            parse_csv(csvtext)
        );
    }
    
    #[test]
    fn test_csvparse_advanced() {
        let csvtext: &str = "Year,Make,Model,Description,Price\r\n1997,Ford,E350,\
                            \"ac, abs, moon\",3000.00\r\n1999,Chevy,\"Venture \"\
                            \"Extended Edition\"\"\",\"\",4900.00\r\n1999,Chevy,\"\
                            Venture \"\"Extended Edition, Very Large\"\"\",\"\",5\
                            000.00\r\n1996,Jeep,Grand Cherokee,\"MUST SELL!\r\nai\
                            r, moon roof, loaded\",4799.00\r\n";
        assert_eq!(Ok(CSV {
                field_len: 5,
                fields: string_vec![
                    "Year", "Make", "Model","Description","Price",
                    "1997", "Ford", "E350","ac, abs, moon","3000.00",
                    "1999", "Chevy", "Venture \"Extended Edition\"","","4900.00",
                    "1999", "Chevy", "Venture \"Extended Edition, Very Large\"","","5000.00",
                    "1996", "Jeep", "Grand Cherokee","MUST SELL!\r\nair, moon roof, loaded","4799.00",
                ],
            }),
            parse_csv(csvtext)
        );
    }

    #[test]
    fn test_csvparse_invalidcharacter() {
        let csvtext: &str = "id,name,em\"ail";
        assert_eq!(Err(ListParserError::CSVIllegalCharacter(10)),
            parse_csv(csvtext)
        );
    }
    
    #[test]
    fn test_csvparse_improperlinebreak() {
        let csvtext: &str = "id,name,email\n1,John,john.doe@example.com";
        assert_eq!(Err(ListParserError::CSVImproperLinebreak(13)),
            parse_csv(csvtext)
        );
    }

    #[test]
    fn test_csvparse_invalidshape() {
        let csvtext: &str = "a,b,c\r\n1,2,3,4";
        assert_eq!(Err(ListParserError::CSVInvalid(14)),
            parse_csv(csvtext)
        );
    }
    
    #[test]
    fn test_csvparse_unclosedquotes() {
        let csvtext: &str = "a,b,c\r\n1,\"2,3";
        assert_eq!(Err(ListParserError::CSVInvalid(13)),
            parse_csv(csvtext)
        );
    }
    
    #[test]
    fn test_csvparse_lonequoute() {
        let csvtext: &str = "a,b,c\r\n1,\"2\"2\",3";
        assert_eq!(Err(ListParserError::CSVInvalid(12)),
            parse_csv(csvtext)
        );
    }

    #[test]
    fn test_shopparse_basic() {
        let csvtext: &str = "Butter,1,ea\r\nCheese,2,kg";
        assert_eq!(Ok(vec![
                ShoppingItemQuery { name: "Butter".to_owned(), quantity: 1, unit: "ea".to_owned()},
                ShoppingItemQuery { name: "Cheese".to_owned(), quantity: 2, unit: "kg".to_owned()},
            ]),
            parse(csvtext)
        );
    }
    
    #[test]
    fn test_shopparse_oneliner() {
        let csvtext: &str = "Butter,1,ea";
        assert_eq!(Ok(vec![
                ShoppingItemQuery { name: "Butter".to_owned(), quantity: 1, unit: "ea".to_owned()}
            ]),
            parse(csvtext)
        );
    }
    
    #[test]
    fn test_shopparse_advanced() {
        let csvtext: &str = "Butter,1,ea\r\n\
                            \"Cheese, American\",2,kg\r\n\
                            \"\"\"Fred\"\"\",1800,mL\r\n\
                            \"Box of Newlines,\n\r\n\n\r\n\"\"100% Organic\",12,$";
        assert_eq!(Ok(vec![
                ShoppingItemQuery { name: "Butter".to_owned(), quantity: 1, unit: "ea".to_owned()},
                ShoppingItemQuery { name: "Cheese, American".to_owned(), quantity: 2, unit: "kg".to_owned()},
                ShoppingItemQuery { name: "\"Fred\"".to_owned(), quantity: 1800, unit: "mL".to_owned()},
                ShoppingItemQuery { name: "Box of Newlines,\n\r\n\n\r\n\"100% Organic".to_owned(), quantity: 12, unit: "$".to_owned()},
            ]),
            parse(csvtext)
        );
    }

    #[test]
    fn test_shopparse_basic_header() {
        let csvtext: &str = "ItemName,Quantity,Unit\r\nButter,1,ea\r\nCheese,2,kg";
        assert_eq!(Ok(vec![
                ShoppingItemQuery { name: "Butter".to_owned(), quantity: 1, unit: "ea".to_owned()},
                ShoppingItemQuery { name: "Cheese".to_owned(), quantity: 2, unit: "kg".to_owned()},
            ]),
            parse(csvtext)
        );
    }
    
    #[test]
    fn test_shopparse_oneliner_header() {
        let csvtext: &str = "ItemName,Quantity,Unit\r\nButter,1,ea";
        assert_eq!(Ok(vec![
                ShoppingItemQuery { name: "Butter".to_owned(), quantity: 1, unit: "ea".to_owned()}
            ]),
            parse(csvtext)
        );
    }
    
    #[test]
    fn test_shopparse_advanced_header() {
        let csvtext: &str = "ItemName,Quantity,Unit\r\n\
                            Butter,1,ea\r\n\
                            \"Cheese, American\",2,kg\r\n\
                            \"\"\"Fred\"\"\",1800,mL\r\n\
                            \"Box of Newlines,\n\r\n\n\r\n\"\"100% Organic\",12,$";
        assert_eq!(Ok(vec![
                ShoppingItemQuery { name: "Butter".to_owned(), quantity: 1, unit: "ea".to_owned()},
                ShoppingItemQuery { name: "Cheese, American".to_owned(), quantity: 2, unit: "kg".to_owned()},
                ShoppingItemQuery { name: "\"Fred\"".to_owned(), quantity: 1800, unit: "mL".to_owned()},
                ShoppingItemQuery { name: "Box of Newlines,\n\r\n\n\r\n\"100% Organic".to_owned(), quantity: 12, unit: "$".to_owned()},
            ]),
            parse(csvtext)
        );
    }
    
}
