use bigdecimal::BigDecimal;
use derive_more::{Add, AddAssign, Sub, SubAssign, Sum};
use serde::{Deserialize, Serialize};

#[derive(
    Default,
    Debug,
    Clone,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Serialize,
    Deserialize,
    Add,
    Sub,
    AddAssign,
    SubAssign,
    Sum,
)]

pub struct Distance {
    base_value_metres: BigDecimal,
}

impl Distance {
    /// if metres is negative, it becomes positive
    pub fn from_metres<T: Into<BigDecimal>>(metres: T) -> Self {
        Self {
            base_value_metres: metres.into().abs(),
        }
    }

    pub fn metres<T: From<BigDecimal>>(&self) -> T {
        self.base_value_metres.clone().into()
    }
}

#[cfg(test)]
mod tests {
    use std::{fmt::Debug, str::FromStr};

    use super::*;

    // assert all elems are eq to all other elems in the slice, checks in both directions, checks that elems are eq to themselves
    fn assert_all_eq(all: &[impl PartialEq + Debug]) {
        for row in all.iter() {
            for col in all.iter() {
                assert_eq!(row, col);
            }
        }
    }

    // assert all elems are ne to all other elems in the slice, checks in both directions
    fn assert_all_ne(all: &[impl PartialEq + Debug]) {
        for (row_i, row) in all.iter().enumerate() {
            for (col_i, col) in all.iter().enumerate() {
                if row_i == col_i {
                    continue;
                }
                assert_ne!(row, col);
            }
        }
    }

    // Clippy is being stupid here, ignore these lints
    fn clone_sort<T: PartialEq + Clone + Ord>(vec: &Vec<T>) -> Vec<T> {
        let mut result = vec.clone();
        result.sort();
        result
    }

    fn vecs() -> (
        Vec<Distance>,
        Vec<Distance>,
        Vec<Distance>,
        Vec<Distance>,
        Vec<Distance>,
        Vec<Distance>,
        Vec<Distance>,
        Vec<Distance>,
        Vec<Distance>,
    ) {
        let a = vec![
            d(798),
            d(710),
            d(2247),
            d(7715),
            d(8110),
            d(2486),
            d(8993),
            d(2924),
            d(6550),
            d(9557),
        ];
        let a_answer = vec![
            d(710),
            d(798),
            d(2247),
            d(2486),
            d(2924),
            d(6550),
            d(7715),
            d(8110),
            d(8993),
            d(9557),
        ];
        let a_subset = vec![d(798), d(710), d(2247), d(7715), d(8110), d(2486)];
        let a_subset_answer = vec![d(710), d(798), d(2247), d(2486), d(7715), d(8110)];

        let a_sort = clone_sort(&a);
        let a_subset_sort = clone_sort(&a_subset);

        let empty = Vec::<Distance>::new();
        let zero = vec![d(0)];
        let zeroes = vec![d(0), d(0), d(0), d(0), d(0), d(0), d(0), d(0), d(0)];

        (
            a,
            a_answer,
            a_subset,
            a_subset_answer,
            a_sort,
            a_subset_sort,
            empty,
            zero,
            zeroes,
        )
    }

    fn ds() -> (Distance,Distance,Distance,Distance,Distance,Distance,Distance,Distance,Distance,Distance,Distance,Distance,Distance,Distance,Distance,Distance,Distance,Distance,Distance,) {
        (
            d(0),
            d(1),
            d(2),
            d(BigDecimal::from_str("1e3").unwrap()),
            d(BigDecimal::from_str("1e6").unwrap()),
            d(BigDecimal::from_str("1e9").unwrap()),
            d(BigDecimal::from_str("1e12").unwrap()),
            d(BigDecimal::from_str("1e15").unwrap()),
            d(BigDecimal::from_str("1e18").unwrap()),
            d(BigDecimal::from_str("1e21").unwrap()),
            d(BigDecimal::from_str("1e-3").unwrap()),
            d(BigDecimal::from_str("1e-6").unwrap()),
            d(BigDecimal::from_str("1e-9").unwrap()),
            d(BigDecimal::from_str("1e-12").unwrap()),
            d(BigDecimal::from_str("1e-15").unwrap()),
            d(BigDecimal::from_str("1e-18").unwrap()),
            d(BigDecimal::from_str("1e-21").unwrap()),
            d(BigDecimal::from_str("1e2e63").unwrap()),
            d(BigDecimal::from_str("1e2e-63").unwrap())
        )
    }

    #[test]
    fn eq() {
        let (a, a_answer, a_subset, a_subset_answer, a_sort, a_subset_sort, empty, zero, zeroes) =
            vecs();

        let (zero, one, two, thou, mil, bil, tril, quad, quin, sex, thouth, milth, bilth, trilth, quadth, quinth, sexth, max, min)

        for vec in [
            a,
            a_answer,
            a_subset,
            a_subset_answer,
            a_sort,
            a_subset_sort,
            empty,
            zero,
            zeroes,
        ]
            assert_eq!(vec, vec);
        
    }

    #[test]
    fn ne() {
        assert_all_ne(
            vec![
                &a,
                &a_sort,
                &a_subset,
                &a_subset_sort,
                &empty,
                &zero,
                &zeroes,
            ]
            .as_slice(),
        );
        assert_all_ne(
            vec![
                &a,
                &a_answer,
                &a_subset,
                &a_subset_sort,
                &empty,
                &zero,
                &zeroes,
            ]
            .as_slice(),
        );
        assert_all_ne(
            vec![
                &a,
                &a_sort,
                &a_subset,
                &a_subset_answer,
                &empty,
                &zero,
                &zeroes,
            ]
            .as_slice(),
        );
        assert_all_ne(
            vec![
                &a,
                &a_answer,
                &a_subset,
                &a_subset_answer,
                &empty,
                &zero,
                &zeroes,
            ]
            .as_slice(),
        );
    }

    #[test]
    fn partial_eq() {}
    #[test]
    fn add() {}
    #[test]
    fn sub() {}
    #[test]
    fn sum() {
        fn t(iter: impl Iterator<Item = Distance>, answer: Distance) {
            assert_eq!(iter.sum::<Distance>(), answer);
        }
    }

    fn d(m: impl Into<BigDecimal>) -> Distance {
        Distance::from_metres(m)
    }
}
