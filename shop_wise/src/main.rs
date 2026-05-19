use std::fs::read_to_string;

use util::store::Store;

use crate::route_planner::filters::{StoreFilters, filter_stores};

mod gui;
mod price_calculator;
mod route_planner;

fn main() {
    let filters: StoreFilters =
        serde_json::from_str(read_to_string("filters.json").unwrap().as_str()).unwrap();

    let stores: Vec<Store> =
        serde_json::from_str(read_to_string("stores.json").unwrap().as_str()).unwrap();

    let output = filter_stores(&stores, &filters);

    print_bar();
    println!("List of all stores:");
    println!("{}", read_to_string("stores.json").unwrap().as_str());

    print_bar();
    println!("List of current user filters:");
    println!("{}", read_to_string("filters.json").unwrap().as_str());

    print_bar();
    println!("Filtered stores:");
    println!("{}", serde_json::to_string_pretty(&output).unwrap());
    print_bar();
}

fn print_bar() {
    let terminal_cols = termsize::get().unwrap().cols;
    for _ in 0..terminal_cols {
        print!("=");
    }
}
