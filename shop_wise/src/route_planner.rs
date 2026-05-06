use std::time::Duration;

use util::cost::Cost;

pub struct Route {
    time: Duration,
    cost: Cost,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
