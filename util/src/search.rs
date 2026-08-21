use crate::{
    search::SearchUnits::{DOLLAR, EACH, GRAM, KILOGRAM, LITRE, MILLILITRE},
    store::{
        Store,
        StoreBrand::{self, Newworld, Paknsave, Woolworths},
    },
};

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum SearchUnits {
    EACH,
    GRAM,
    KILOGRAM,
    MILLILITRE,
    LITRE,
    DOLLAR,
}

pub fn measurement_type(unit: SearchUnits) -> &'static str {
    match unit {
        EACH => "count",
        GRAM => "mass",
        KILOGRAM => "mass",
        MILLILITRE => "volume",
        LITRE => "volume",
        DOLLAR => "cost",
    }
}

pub fn unit_to_str(unit: SearchUnits) -> &'static str {
    match unit {
        EACH => "ea",
        GRAM => "g",
        KILOGRAM => "kg",
        MILLILITRE => "mL",
        LITRE => "L",
        DOLLAR => "$",
    }
}

pub fn match_to_unit(name: &str) -> Option<SearchUnits> {
    match name {
        "ea" => Some(EACH),
        "g" => Some(GRAM),
        "kg" => Some(KILOGRAM),
        "mL" => Some(MILLILITRE),
        "L" => Some(LITRE),
        "$" => Some(DOLLAR),
        _ => None,
    }
}

pub fn match_sid_to_brand(sid: u32) -> Option<StoreBrand> {
    match (sid) {
        1 => Some(Paknsave),
        2 => Some(Newworld),
        3 => Some(Woolworths),
        _ => None,
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct ShoppingItemQuery {
    pub name: String,
    pub quantity: u32,
    pub unit: String,
}

#[derive(Debug, PartialEq)]
pub struct ShoppingItem {
    pub name: String,
    pub quantity: u32,
    pub unit: SearchUnits,
    pub price: u32,
    pub store: Store,
}

#[cfg(test)]
mod test {
    use crate::search::{SearchUnits, match_to_unit, unit_to_str};

    #[test]
    fn test_symettric() {
        assert_eq!(
            match_to_unit(unit_to_str(SearchUnits::EACH)),
            Some(SearchUnits::EACH)
        );
        assert_eq!(
            match_to_unit(unit_to_str(SearchUnits::GRAM)),
            Some(SearchUnits::GRAM)
        );
        assert_eq!(
            match_to_unit(unit_to_str(SearchUnits::KILOGRAM)),
            Some(SearchUnits::KILOGRAM)
        );
        assert_eq!(
            match_to_unit(unit_to_str(SearchUnits::MILLILITRE)),
            Some(SearchUnits::MILLILITRE)
        );
        assert_eq!(
            match_to_unit(unit_to_str(SearchUnits::LITRE)),
            Some(SearchUnits::LITRE)
        );
        assert_eq!(
            match_to_unit(unit_to_str(SearchUnits::DOLLAR)),
            Some(SearchUnits::DOLLAR)
        );
    }
}
