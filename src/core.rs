use rust_decimal::Decimal;
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Instrument(pub String);

impl Instrument {
    #[allow(dead_code)]
    pub fn new(s: String) -> Self {
        Self(s.to_string())
    }
}

impl From<String> for Instrument {
    fn from(value: String) -> Self {
        Self(value)
    }
}
impl From<&str> for Instrument {
    fn from(value: &str) -> Self {
        Self(value.to_string())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Price(pub Decimal);

impl Price {
    pub fn new(d: Decimal) -> Self {
        Self(d)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Quantity(pub i64);

impl Quantity {
    pub fn new(q: i64) -> Self {
        Self(q)
    }
}

impl std::ops::Neg for Quantity {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self(-self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Side {
    Buy,
    Sell,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrderType {
    Limit,
    Stop,
}

#[derive(Debug, Clone)]
pub struct MarketData {
    pub bid: Price,
    pub ask: Price,
}

impl MarketData {
    pub fn new(bid: Price, ask: Price) -> Self {
        Self { bid, ask }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Position {
    pub instrument: Instrument,
    pub last_qty: Quantity, // Positions can be negative for short
    pub last_px: Price,
}

impl Position {
    pub fn new(instrument: Instrument, qty: Quantity, price: Price) -> Self {
        Self {
            instrument,
            last_qty: qty,
            last_px: price,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Order {
    pub id: Uuid,
    pub side: Side,
    pub qty: Quantity,
    pub instrument: Instrument,
    pub order_type: OrderType,
    pub px: Price,
    pub oco_with: Vec<Uuid>,
}

impl Order {
    pub fn new(
        side: Side,
        qty: Quantity,
        instrument: Instrument,
        order_type: OrderType,
        price: Price,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            side,
            qty,
            instrument,
            order_type,
            px: price,
            oco_with: Vec::new(),
        }
    }

    /// *one cancels the other* makes the orders mutually exclusive so that executing one cancels the other.
    pub fn make_oco(orders: Vec<Order>) -> Vec<Order> {
        let ids: Vec<Uuid> = orders.iter().map(|o| o.id).collect();
        orders
            .into_iter()
            .map(|mut o| {
                o.oco_with = ids.clone();
                o
            })
            .collect()
    }
}

#[derive(Debug, Clone)]
pub struct Trade {
    pub instrument: Instrument,
    pub last_qty: Quantity,
    pub last_px: Price,
}

impl Trade {
    pub fn new(instrument: Instrument, qty: Quantity, price: Price) -> Self {
        Self {
            instrument,
            last_qty: qty,
            last_px: price,
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct State {
    pub trades: Vec<Trade>,
    pub positions: HashMap<Instrument, Position>,
    pub market: HashMap<Instrument, MarketData>,
    pub open_orders: Vec<Order>,
}

impl State {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self::default()
    }
}

// ----------------------------------------------------------------
// Market
// ----------------------------------------------------------------

#[allow(dead_code)]
pub fn get_market(state: &State, cross: &Instrument) -> MarketData {
    state
        .market
        .get(cross)
        .cloned()
        .expect(&format!("Market not found for instrument: {:?}", cross))
}

#[allow(dead_code)]
pub fn set_market(mut state: State, cross: Instrument, bid: Price, ask: Price) -> State {
    state.market.insert(cross, MarketData::new(bid, ask));
    state
}

// ----------------------------------------------------------------
// Positions
// ----------------------------------------------------------------

#[allow(dead_code)]
pub fn clear_positions(mut state: State) -> State {
    state.positions.clear();
    state
}

#[allow(dead_code)]
pub fn get_positions(state: &State) -> HashMap<Instrument, Position> {
    state.positions.clone()
}

#[allow(dead_code)]
pub fn get_position(state: &State, instrument: &Instrument) -> Option<Position> {
    state.positions.get(instrument).cloned()
}

#[allow(dead_code)]
pub fn set_position(
    mut state: State,
    instrument: Instrument,
    qty: Quantity,
    price: Price,
) -> State {
    state
        .positions
        .insert(instrument.clone(), Position::new(instrument, qty, price));
    state
}

// ----------------------------------------------------------------
// Open orders
// ----------------------------------------------------------------

pub fn clear_open_orders(mut state: State) -> State {
    state.open_orders.clear();
    state
}

pub fn remove_open_orders<F>(mut state: State, pred: F) -> State
where
    F: Fn(&Order) -> bool,
{
    state.open_orders.retain(|o| !pred(o));
    state
}

pub fn filter_open_orders<F>(state: &State, pred: F) -> Vec<Order>
where
    F: Fn(&Order) -> bool,
{
    state
        .open_orders
        .iter()
        .filter(|&o| pred(o))
        .cloned()
        .collect()
}

pub fn submit_orders(mut state: State, orders: Vec<Order>) -> State {
    state.open_orders.extend(orders);
    state
}

pub fn submit_oco_orders(state: State, a: Order, b: Order) -> State {
    submit_orders(state, Order::make_oco(vec![a, b]))
}

// ----------------------------------------------------------------
// Trades
// ----------------------------------------------------------------

pub fn clear_trades(mut state: State) -> State {
    state.trades.clear();
    state
}

pub fn get_trades(state: &State) -> Vec<Trade> {
    state.trades.clone()
}

pub fn register_trade(
    mut state: State,
    instrument: Instrument,
    qty: Quantity,
    price: Price,
) -> State {
    state.trades.push(Trade::new(instrument, qty, price));
    state
}

fn trade(mut state: State, cross: Instrument, qty: Quantity, price: Price) -> State {
    state.trades.push(Trade::new(cross.clone(), qty, price));
    state
        .positions
        .insert(cross.clone(), Position::new(cross, qty, price));
    state
}

pub fn buy(state: State, cross: Instrument, qty: Quantity) -> State {
    let market_data = get_market(&state, &cross);
    let price = market_data.ask;
    trade(state, cross, qty, price)
}

pub fn buy_with_orders(
    state: State,
    cross: Instrument,
    qty: Quantity,
    target: Price,
    stop: Price,
) -> State {
    let take_profit = Order::new(Side::Sell, qty, cross.clone(), OrderType::Limit, target);
    let stop_loss = Order::new(Side::Sell, qty, cross.clone(), OrderType::Stop, stop);

    let market_data = get_market(&state, &cross);
    let price = market_data.ask;

    let state = trade(state, cross, qty, price);

    let oco_orders = Order::make_oco(vec![take_profit, stop_loss]);
    submit_orders(state, oco_orders)
}

pub fn sell(state: State, cross: Instrument, qty: Quantity) -> State {
    let market_data = get_market(&state, &cross);
    let price = market_data.bid;
    trade(state, cross, -qty, price)
}

// ----------------------------------------------------------------
// Tests
// ----------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_get_market() {
        let state = State::new();
        let instrument = Instrument::from("EURUSD");
        let state = set_market(
            state,
            instrument.clone(),
            Price(dec!(1.34662)),
            Price(dec!(1.34714)),
        );
        let m = get_market(&state, &instrument);
        assert_eq!(m.bid, Price(dec!(1.34662)));
        assert_eq!(m.ask, Price(dec!(1.34714)));
    }

    #[test]
    fn test_trade_list_manipulation() {
        let state = State::new();
        let instrument = Instrument::from("EURUSD");

        // is (empty? (get-trades))
        assert!(get_trades(&state).is_empty());

        // (register-trade! "EURUSD" 1000000 1.34714)
        let state = register_trade(
            state,
            instrument.clone(),
            Quantity(1000000),
            Price(dec!(1.34714)),
        );

        let trades = get_trades(&state);
        assert_eq!(trades.len(), 1);
        assert_eq!(trades[0].instrument, instrument);
        assert_eq!(trades[0].last_qty, Quantity(1000000));
        assert_eq!(trades[0].last_px, Price(dec!(1.34714)));
    }

    #[test]
    fn test_buy_at_market() {
        let state = State::new();
        let instrument = Instrument::from("EURUSD");
        let state = set_market(
            state,
            instrument.clone(),
            Price(dec!(1.34662)),
            Price(dec!(1.34714)),
        );

        // (buy! "EURUSD" 1000000)
        let state = buy(state, instrument.clone(), Quantity(1000000));

        let trades = get_trades(&state);
        let positions = get_positions(&state);
        let eurusd_pos = get_position(&state, &instrument).unwrap();

        assert_eq!(trades.len(), 1);
        assert_eq!(trades[0].instrument, instrument);
        assert_eq!(trades[0].last_qty, Quantity(1000000));
        assert_eq!(trades[0].last_px, Price(dec!(1.34714)));

        assert_eq!(positions.len(), 1);
        assert_eq!(eurusd_pos.instrument, instrument);
        assert_eq!(eurusd_pos.last_qty, Quantity(1000000));
        assert_eq!(eurusd_pos.last_px, Price(dec!(1.34714)));
    }

    #[test]
    fn test_buy_with_limit_and_stop_clojure() {
        let state = State::new();
        let instrument = Instrument::from("EURUSD");
        let state = set_market(
            state,
            instrument.clone(),
            Price(dec!(1.34662)),
            Price(dec!(1.34714)),
        );

        // (buy! "EURUSD" 1000000 1.4000 1.3000)
        let state = buy_with_orders(
            state,
            instrument.clone(),
            Quantity(1000000),
            Price(dec!(1.4000)),
            Price(dec!(1.3000)),
        );

        let trades = get_trades(&state);
        let positions = get_positions(&state);
        let eurusd_pos = get_position(&state, &instrument).unwrap();
        let eurusd_orders = filter_open_orders(&state, |o| o.instrument == instrument);

        assert_eq!(trades.len(), 1);
        assert_eq!(trades[0].last_qty, Quantity(1000000));
        assert_eq!(trades[0].last_px, Price(dec!(1.34714)));

        assert_eq!(positions.len(), 1);
        assert_eq!(eurusd_pos.last_qty, Quantity(1000000));

        assert_eq!(eurusd_orders.len(), 2);

        let limit = eurusd_orders
            .iter()
            .find(|o| o.order_type == OrderType::Limit)
            .expect("Limit order not found");
        let stop = eurusd_orders
            .iter()
            .find(|o| o.order_type == OrderType::Stop)
            .expect("Stop order not found");

        assert_eq!(limit.side, Side::Sell);
        assert_eq!(limit.qty, Quantity(1000000));
        assert_eq!(limit.instrument, instrument);
        assert_eq!(limit.px, Price(dec!(1.4000)));

        assert_eq!(stop.side, Side::Sell);
        assert_eq!(stop.qty, Quantity(1000000));
        assert_eq!(stop.instrument, instrument);
        assert_eq!(stop.px, Price(dec!(1.3000)));
    }

    #[test]
    fn test_sell_at_market() {
        let state = State::new();
        let instrument = Instrument::from("EURUSD");
        let state = set_market(
            state,
            instrument.clone(),
            Price(dec!(1.34662)),
            Price(dec!(1.34714)),
        );

        // (sell! "EURUSD" 1000000)
        let state = sell(state, instrument.clone(), Quantity(1000000));

        let trades = get_trades(&state);
        let positions = get_positions(&state);
        let eurusd_pos = get_position(&state, &instrument).unwrap();

        assert_eq!(trades.len(), 1);
        assert_eq!(trades[0].instrument, instrument);
        assert_eq!(trades[0].last_qty, Quantity(-1000000));
        assert_eq!(trades[0].last_px, Price(dec!(1.34662)));

        assert_eq!(positions.len(), 1);
        assert_eq!(eurusd_pos.instrument, instrument);
        assert_eq!(eurusd_pos.last_qty, Quantity(-1000000));
        assert_eq!(eurusd_pos.last_px, Price(dec!(1.34662)));
    }

    #[test]
    fn test_quantity_neg() {
        let q = Quantity(100);
        assert_eq!(-q, Quantity(-100));
        assert_eq!(-(-q), Quantity(100));
    }
}
