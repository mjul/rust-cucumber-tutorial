use cucumber::{World, given};
use cuketut::core::{Instrument, Position, Price, Quantity, State};
use rust_decimal::prelude::*;

// The `World` is your shared, likely mutable state.
// Cucumber constructs it via `Default::default()` for each scenario.
#[derive(Debug, Default, World)]
pub struct TradingWorld {
    state: State,
}

impl TradingWorld {
    pub fn map_state<F>(&mut self, f: F) -> &mut Self
    where
        F: FnOnce(State) -> State,
    {
        let old = std::mem::replace(&mut self.state, State::default());
        self.state = f(old);
        self
    }
}

// Steps are defined with `given`, `when` and `then` attributes.

fn set_position(world: &mut TradingWorld, cross: String, qty: i64, price: String) {
    let px = Price(Decimal::from_str(price.as_str()).expect("price cannot be parsed as decimal"));
    world.map_state(move |state| {
        cuketut::core::set_position(state, Instrument::from(cross), Quantity(qty), px)
    });
}

// English step definition
#[given(regex = r"^that my position in (\w{6}) is (\d+) at ([\d.]+)$")]
async fn my_initial_position_is_en(
    world: &mut TradingWorld,
    cross: String,
    qty: i64,
    price: String,
) {
    set_position(world, cross, qty, price);
}

// Danish step definition (handling comma decimal separator)
#[given(regex = r"^at min position i (\w{6}) er (\d+) købt til kurs ([\d,]+)$")]
async fn my_initial_position_is_dk(
    world: &mut TradingWorld,
    cross: String,
    qty: i64,
    price: String,
) {
    // map between comma and period as decimal separators
    let en_price = price.replace(',', ".").to_string();
    set_position(world, cross, qty, en_price);
    println!("{:?}", world);
}

// This runs before everything else, so you can set up things here.
#[tokio::main]
async fn main() {
    // You may choose any executor you like (`tokio`, `async-std`, etc.).
    // I use tokio out of habit
    TradingWorld::run("features/open_position.feature").await;
    // TODO: run "features/open_position_da.feature",
    // TODO: run "features/conditional_order.feature",
}
