use cucumber::gherkin::{Step, Table};
use cucumber::{World, given, then, when};
use cuketut::core::{Instrument, OrderType, Position, Price, Quantity, Side, State};
use rust_decimal::prelude::*;
use std::collections::HashMap;

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

fn my_position_should_show_long(world: &TradingWorld, qty: i64, cross: String, price: Price) {
    let inst = Instrument::from(cross.clone());
    let actual = cuketut::core::get_position(&world.state, &inst)
        .expect("expected to find position for instrument");
    let expected = Position::new(inst.clone(), Quantity::new(qty), price);
    assert_eq!(expected, actual);
}

#[then(regex = r"^my position should show LONG (\d+) (\w{6}) at ([\d.]+)$")]
fn my_position_should_show_long_en(
    world: &mut TradingWorld,
    qty: i64,
    cross: String,
    price: String,
) {
    my_position_should_show_long(world, qty, cross, parse_price_en(&price));
}

#[then(regex = r"^min position skal være LANG (\d+) (\w{6}) købt til kurs ([\d,]+)$")]
fn my_position_should_show_long_da(
    world: &mut TradingWorld,
    qty: i64,
    cross: String,
    price: String,
) {
    my_position_should_show_long(world, qty, cross, parse_price_da(&price));
}

fn my_position_should_show_short(world: &TradingWorld, qty: i64, cross: String, price: Price) {
    let inst = Instrument::from(cross.clone());
    let actual = cuketut::core::get_position(&world.state, &inst)
        .expect("expected to find position for instrument");
    let expected = Position::new(inst.clone(), -Quantity::new(qty), price);
    assert_eq!(expected, actual);
}

#[then(regex = r"^my position should show SHORT (\d+) (\w{6}) at ([\d.]+)$")]
fn my_position_should_show_short_en(
    world: &mut TradingWorld,
    qty: i64,
    cross: String,
    price: String,
) {
    my_position_should_show_short(world, qty, cross, parse_price_en(&price));
}

#[then(regex = r"^min position skal være KORT (\d+) (\w{6}) solgt til kurs ([\d,]+)$")]
fn my_position_should_show_short_da(
    world: &mut TradingWorld,
    qty: i64,
    cross: String,
    price: String,
) {
    my_position_should_show_short(world, qty, cross, parse_price_da(&price));
}

// We don't have to use a separate function, we can put the #[given] on the implementation
#[given(regex = r"^I have no open orders in (\w{6})$")]
fn i_have_no_open_orders_in(world: &mut TradingWorld, cross: String) {
    let inst = Instrument::from(cross);
    world.map_state(|s| cuketut::core::remove_open_orders(s, |o| o.instrument == inst));
}

#[when(
    regex = r"^I submit an order to BUY (\d+) (\w{6}) at MKT with TARGET ([\d.]+) and STOP ([\d.]+)$"
)]
fn i_submit_an_order_to_buy_at_market_with_target_and_stop(
    world: &mut TradingWorld,
    qty: i64,
    cross: String,
    target: String,
    stop: String,
) {
    world.map_state(move |state| {
        cuketut::core::buy_with_orders(
            state,
            Instrument::from(cross),
            Quantity::new(qty),
            parse_price_en(&target),
            parse_price_en(&stop),
        )
    });
}

/// Translate a table to a vector of `HashMap`, one for each row, where the keys are the
/// column names from the header row and the values are the values in the data row.
fn table_to_hash_maps(table: &Table) -> Vec<HashMap<String, String>> {
    match table.rows.as_slice() {
        [] => vec![],
        [_headers] => vec![],
        [headers, data @ ..] => {
            // The table is a list of rows, every row is a list of fields (strings)
            // Translate it to a list of keyed maps (one per row, excluding the header),
            // using the headline value as the key for each field
            data.iter()
                .map(|row| {
                    headers
                        .iter()
                        .cloned()
                        .zip(row.iter().cloned())
                        .collect::<HashMap<String, String>>()
                })
                .collect()
        }
    }
}

// Using a data table, see https://cucumber-rs.github.io/cucumber/current/writing/data_tables.html
#[then(regex = r"^my open orders should contain these OCO-orders$")]
fn my_open_orders_should_contain_these_oco_orders(world: &mut TradingWorld, step: &Step) {
    if let Some(table) = step.table.as_ref() {
        let data = table_to_hash_maps(table);
        for datum in data {
            let side_str = datum
                .get("Side")
                .expect("Side column is missing in the table");
            let side = match side_str.as_str() {
                "BUY" => Some(Side::Buy),
                "SELL" => Some(Side::Sell),
                _ => None,
            }
            .expect("Side must be BUY or SELL");
            let qty_str = datum
                .get("Quantity")
                .expect("Quantity column is missing in the table");
            let qty = Quantity::new(qty_str.parse::<i64>().expect("Quantity must be i64"));
            let cross_str = datum
                .get("Cross")
                .expect("Cross column is missing in the table");
            let inst = Instrument::from(cross_str.to_string());
            let ot_string = datum
                .get("Type")
                .expect("Type column is missing in the table");
            let ot = match ot_string.as_str() {
                "LIMIT" => Some(OrderType::Limit),
                "STOP" => Some(OrderType::Stop),
                _ => None,
            }
            .expect("Type must be LIMIT or STOP");
            let px_string = datum
                .get("Price")
                .expect("Price column is missing in the table");
            let px = parse_price_en(&px_string);

            let matching_orders = cuketut::core::filter_open_orders(&world.state, |o| {
                o.side == side
                    && o.qty == qty
                    && o.instrument == inst
                    && o.order_type == ot
                    && o.px == px
            });

            // Show the expected values, the matches and the total list of open orders on error to aid troubleshooting
            assert_eq!(
                1,
                matching_orders.len(),
                "Expected one matching order for {:?}, found {:?} in {:?}",
                datum,
                matching_orders,
                cuketut::core::filter_open_orders(&world.state, |_| true)
            );
        }
    }
}

// This runs before everything else, so you can set up things here.
#[tokio::main]
async fn main() {
    // You may choose any executor you like (`tokio`, `async-std`, etc.).
    // I use tokio out of habit
    TradingWorld::cucumber()
        .run_and_exit("features/")  // default: run all features
        .await;
}
