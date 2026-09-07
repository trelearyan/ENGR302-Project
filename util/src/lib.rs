use num_traits::Float;

pub mod coordinate;
pub mod cost;
pub mod distance;
pub mod search;
pub mod speed;
pub mod store;

#[cfg(test)]
mod test {}

/// margin between 0 and 1. Below 0 is 0, above 1 is 1.
pub fn float_absolute_compare<T: Float>(a: T, b: T, margin: T) -> bool {
    (a - b).abs() < margin
}
