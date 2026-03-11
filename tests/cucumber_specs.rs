use cucumber::{World, given, then, when};
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

/// Parse an English format price with period as decimal separator
fn parse_price_en(s: &str) -> Price {
    Price::new(Decimal::from_str(s).expect("price cannot be parsed as decimal"))
}

/// Parse a Danish format price with comma as decimal separator
fn parse_price_da(s: &str) -> Price {
    let english = s.replace(',', ".");
    parse_price_en(&english)
}

// Steps are defined with `given`, `when` and `then` attributes.

// We can share step definitions by using normal functions like this
fn my_initial_position_is(world: &mut TradingWorld, cross: String, qty: i64, price: Price) {
    world.map_state(move |state| {
        cuketut::core::set_position(state, Instrument::from(cross), Quantity(qty), price)
    });
}

// English step definition
#[given(regex = r"^that my position in (\w{6}) is (\d+) at ([\d.]+)$")]
fn my_initial_position_is_en(world: &mut TradingWorld, cross: String, qty: i64, price: String) {
    my_initial_position_is(world, cross, qty, parse_price_en(&price));
}

// Danish step definition (handling comma decimal separator)
#[given(regex = r"^at min position i (\w{6}) er (\d+) købt til kurs ([\d,]+)$")]
fn my_initial_position_is_da(world: &mut TradingWorld, cross: String, qty: i64, price: String) {
    my_initial_position_is(world, cross, qty, parse_price_da(&price));
}

#[given(regex = r"^the market for (\w{6}) is at \[([\d.]+);([\d.]+)\]$")]
fn the_market_is_at_en(world: &mut TradingWorld, cross: String, bid: String, ask: String) {
    world.map_state(move |state| {
        cuketut::core::set_market(
            state,
            Instrument::from(cross),
            parse_price_en(&bid),
            parse_price_en(&ask),
        )
    });
}

#[given(regex = r"^markedsprisen for (\w{6}) er \[([\d,]+);([\d,]+)\]$")]
fn the_market_is_at_da(world: &mut TradingWorld, cross: String, bid: String, ask: String) {
    world.map_state(move |state| {
        cuketut::core::set_market(
            state,
            Instrument::from(cross),
            parse_price_da(&bid),
            parse_price_da(&ask),
        )
    });
}

fn i_submit_an_order_to_buy_at_market(world: &mut TradingWorld, qty: i64, cross: String) {
    world.map_state(move |state| {
        cuketut::core::buy(state, Instrument::from(cross), Quantity::new(qty))
    });
}

// We can add multiple givens, whens or thens to a function
#[when(regex = r"^I submit an order to BUY (\d+) (\w{6}) at MKT$")]
#[when(regex = r"^jeg afgiver en ordre om at KØBE (\d+) (\w{6}) til MARKEDSPRIS$")]
fn i_submit_an_order_to_buy_at_market_en(world: &mut TradingWorld, qty: i64, cross: String) {
    i_submit_an_order_to_buy_at_market(world, qty, cross);
}

fn i_submit_an_order_to_sell_at_market(world: &mut TradingWorld, qty: i64, cross: String) {
    world.map_state(move |state| {
        cuketut::core::sell(state, Instrument::from(cross), Quantity::new(qty))
    });
}

#[when(regex = r"^I submit an order to SELL (\d+) (\w{6}) at MKT$")]
fn i_submit_an_order_to_sell_at_market_en(world: &mut TradingWorld, qty: i64, cross: String) {
    i_submit_an_order_to_sell_at_market(world, qty, cross);
}

fn a_trade_should_be_made_at(world: &TradingWorld, expected: Price) {
    let actual = cuketut::core::get_trades(&world.state)
        .last()
        .unwrap()
        .last_px;
    assert_eq!(expected, actual);
}

#[then(regex = r"^a trade should be made at ([\d.]+)$")]
fn a_trade_should_be_made_en(world: &mut TradingWorld, price: String) {
    a_trade_should_be_made_at(world, parse_price_en(&price));
}

#[then(regex = r"^skal en handel ske til kurs ([\d,]+)$")]
fn a_trade_should_be_made_da(world: &mut TradingWorld, price: String) {
    a_trade_should_be_made_at(world, parse_price_da(&price));
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
