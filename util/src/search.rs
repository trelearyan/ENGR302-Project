use serde::{Deserialize, Serialize};

use crate::{
    cost::Cost,
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

impl SearchUnits {
    /// Gives the physical quantity that the unit is measuring (e.g. count, mass, volume, cost)
    pub fn measurement_type(&self) -> &'static str {
        match self {
            EACH => "count",
            GRAM => "mass",
            KILOGRAM => "mass",
            MILLILITRE => "volume",
            LITRE => "volume",
            DOLLAR => "cost",
        }
    }

    /// Gives the string representation of the unit
    pub fn to_str(&self) -> &'static str {
        match self {
            EACH => "ea",
            GRAM => "g",
            KILOGRAM => "kg",
            MILLILITRE => "mL",
            LITRE => "L",
            DOLLAR => "$",
        }
    }

    /// Matches the string representation to the unit
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

    /// Called on a search term to find the multiplier for an item.
    /// Will return at least the searched quantity
    /// (so rounding up), but for incompatible units (like LITRE and EACH)
    /// may return less than desired due to missing the information for that item
    /// 
    /// For example, if the user is searching for 6ea, and an item is
    /// available in 4 x 2pk (so 8ea amounts), it will round up to 8ea
    /// and so return 1 (that is EACH.scale_to_match(6, EACH, 8) -> 1)
    /// which means 8*1 -> 8 so choose 8 of that item and multiply price by 1
    /// 
    /// For units with physical meanings like weight or volume,
    /// it will perform unit conversion 
    /// (LITRE.scale_to_match(3, MILLILITRE, 2500) -> 2)
    /// which means 2500*2 -> 5000, and price multiply by 2
    /// 
    /// For dollar rounding down (not below 1) so the price
    /// stays under your limit
    /// 
    /// If no conversion can be found, None will be returned
    pub fn scale_to_match(&self, search_quantity: u32, base_unit: &SearchUnits, base_quantity: u32, cost_cents: u32) -> Option<u32> {
        match self {
            EACH => {
                return match base_unit {
                    EACH => Some((search_quantity + base_quantity - 1) / base_quantity),
                    _ => None,
                };
            },
            GRAM => {
                return match base_unit {
                    KILOGRAM => Some((search_quantity + base_quantity * 1000 - 1) / (base_quantity * 1000)),
                    GRAM => Some((search_quantity + base_quantity - 1) / base_quantity),
                    _ => None,
                }
            },
            KILOGRAM => {
                return match base_unit {
                    GRAM => Some((search_quantity * 1000 + base_quantity - 1) / base_quantity),
                    KILOGRAM => Some((search_quantity + base_quantity - 1) / base_quantity),
                    _ => None,
                }
            },
            MILLILITRE => {
                return match base_unit {
                    LITRE => Some((search_quantity + base_quantity * 1000 - 1) / (base_quantity * 1000)),
                    MILLILITRE => Some((search_quantity + base_quantity - 1) / base_quantity),
                    _ => None,
                }
            },
            LITRE => {
                return match base_unit {
                    MILLILITRE => Some((search_quantity * 1000 + base_quantity - 1) / base_quantity),
                    LITRE => Some((search_quantity + base_quantity - 1) / base_quantity),
                    _ => None,
                }
            },
            DOLLAR => {
                let mul = (search_quantity * 100) / cost_cents;
                return if (mul >= 1) {
                    Some(mul)
                } else {
                    None
                }
            },
        };
    }
}

#[must_use]
pub fn match_sid_to_brand(sid: u32) -> Option<StoreBrand> {
    match sid  {
        1 => Some(Paknsave),
        2 => Some(Woolworths),
        3 => Some(Newworld),
        _ => None,
    }
}

#[derive(Debug, Eq, PartialEq, Clone, Serialize, Deserialize)]
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
    pub price: Cost,
    pub store: Store,
}

#[cfg(test)]
mod test {
    use crate::search::SearchUnits::{self, EACH};

    #[test]
    fn test_symettric() {
        assert_eq!(
            SearchUnits::match_to_unit(SearchUnits::EACH.to_str()),
            Some(SearchUnits::EACH)
        );
        assert_eq!(
            SearchUnits::match_to_unit(SearchUnits::GRAM.to_str()),
            Some(SearchUnits::GRAM)
        );
        assert_eq!(
            SearchUnits::match_to_unit(SearchUnits::KILOGRAM.to_str()),
            Some(SearchUnits::KILOGRAM)
        );
        assert_eq!(
            SearchUnits::match_to_unit(SearchUnits::MILLILITRE.to_str()),
            Some(SearchUnits::MILLILITRE)
        );
        assert_eq!(
            SearchUnits::match_to_unit(SearchUnits::LITRE.to_str()),
            Some(SearchUnits::LITRE)
        );
        assert_eq!(
            SearchUnits::match_to_unit(SearchUnits::DOLLAR.to_str()),
            Some(SearchUnits::DOLLAR)
        );
    }

    #[test]
    fn test_scaling() {
        // Test EACH scaling
        assert_eq!(
            SearchUnits::EACH.scale_to_match(6, &SearchUnits::EACH, 8, 100),
            Some(1)
        );
        // Test Physical scaling
        assert_eq!(
            SearchUnits::LITRE.scale_to_match(3, &SearchUnits::MILLILITRE, 2500, 100),
            Some(2)
        );
        assert_eq!(
            SearchUnits::MILLILITRE.scale_to_match(3000, &SearchUnits::LITRE, 2, 100),
            Some(2)
        );
        assert_eq!(
            SearchUnits::LITRE.scale_to_match(4, &SearchUnits::LITRE, 2, 100),
            Some(2)
        );
        assert_eq!(
            SearchUnits::LITRE.scale_to_match(5, &SearchUnits::LITRE, 2, 100),
            Some(3)
        );
        assert_eq!(
            SearchUnits::KILOGRAM.scale_to_match(3, &SearchUnits::GRAM, 2500, 100),
            Some(2)
        );
        assert_eq!(
            SearchUnits::GRAM.scale_to_match(3000, &SearchUnits::KILOGRAM, 2, 100),
            Some(2)
        );
        assert_eq!(
            SearchUnits::KILOGRAM.scale_to_match(4, &SearchUnits::KILOGRAM, 2, 100),
            Some(2)
        );
        assert_eq!(
            SearchUnits::KILOGRAM.scale_to_match(5, &SearchUnits::KILOGRAM, 2, 100),
            Some(3)
        );
        // Test Dollar scaling
        assert_eq!(
            SearchUnits::DOLLAR.scale_to_match(3, &SearchUnits::MILLILITRE, 2500, 100),
            Some(3)
        );
        assert_eq!(
            SearchUnits::DOLLAR.scale_to_match(8, &SearchUnits::EACH, 1, 780),
            Some(1)
        );
        assert_eq!(
            SearchUnits::DOLLAR.scale_to_match(20, &SearchUnits::EACH, 1, 780),
            Some(2)
        );
        assert_eq!(
            SearchUnits::DOLLAR.scale_to_match(8, &SearchUnits::EACH, 1, 860),
            None
        );
        // Test invalid scaling
        assert_eq!(
            SearchUnits::LITRE.scale_to_match(3, &SearchUnits::EACH, 2500, 100),
            None
        );
    }
}
