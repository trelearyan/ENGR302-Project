use std::array;

mod gui;
mod price_calculator;
mod route_planner;

fn main() {
    let a = "hheahea".to_string();
    let b = a.clone();
    let c: [String; 4928] = array::from_fn(|_| "hehe".to_string());
    let d = c.clone();
    println!("Hello, world!");
}
