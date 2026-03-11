use rust_decimal::Decimal;
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Instrument(pub String);

impl Instrument {
    #[allow(dead_code)]
    pub fn new(s: &str) -> Self {
        Self(s.to_string())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Price(pub Decimal);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Quantity(pub i64);

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

#[derive(Debug, Clone)]
pub struct Position {
    pub instrument: Instrument,
    pub last_qty: Quantity, // Positions can be negative for short
    pub last_px: Price,
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

#[derive(Debug, Clone)]
pub struct Trade {
    pub instrument: Instrument,
    pub last_qty: Quantity,
    pub last_px: Price,
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
    state.market.insert(cross, MarketData { bid, ask });
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
        .insert(instrument.clone(), make_position(instrument, qty, price));
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
    submit_orders(state, make_oco(vec![a, b]))
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
    state.trades.push(make_trade(instrument, qty, price));
    state
}

fn trade(mut state: State, cross: Instrument, qty: Quantity, price: Price) -> State {
    state.trades.push(make_trade(cross.clone(), qty, price));
    state
        .positions
        .insert(cross.clone(), make_position(cross, qty, price));
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
    let take_profit = create_order(Side::Sell, qty, cross.clone(), OrderType::Limit, target);
    let stop_loss = create_order(Side::Sell, qty, cross.clone(), OrderType::Stop, stop);

    let market_data = get_market(&state, &cross);
    let price = market_data.ask;

    let state = trade(state, cross, qty, price);

    let oco_orders = make_oco(vec![take_profit, stop_loss]);
    submit_orders(state, oco_orders)
}

pub fn sell(state: State, cross: Instrument, qty: Quantity) -> State {
    let market_data = get_market(&state, &cross);
    let price = market_data.bid;
    trade(state, cross, -qty, price)
}

// ----------------------------------------------------------------
// Helper functions
// ----------------------------------------------------------------

pub fn make_position(instrument: Instrument, qty: Quantity, price: Price) -> Position {
    Position {
        instrument,
        last_qty: qty,
        last_px: price,
    }
}

pub fn create_order(
    side: Side,
    qty: Quantity,
    instrument: Instrument,
    order_type: OrderType,
    price: Price,
) -> Order {
    Order {
        id: Uuid::new_v4(),
        side,
        qty,
        instrument,
        order_type,
        px: price,
        oco_with: Vec::new(),
    }
}

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

pub fn make_trade(instrument: Instrument, qty: Quantity, price: Price) -> Trade {
    Trade {
        instrument,
        last_qty: qty,
        last_px: price,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_market() {
        let state = State::new();
        let instrument = Instrument::new("EURUSD");
        let state = set_market(
            state,
            instrument.clone(),
            Price(dec!(1.1000)),
            Price(dec!(1.1005)),
        );
        let m = get_market(&state, &instrument);
        assert_eq!(m.bid, Price(dec!(1.1000)));
        assert_eq!(m.ask, Price(dec!(1.1005)));
    }

    #[test]
    fn test_buy() {
        let state = State::new();
        let instrument = Instrument::new("EURUSD");
        let state = set_market(
            state,
            instrument.clone(),
            Price(dec!(1.1000)),
            Price(dec!(1.1005)),
        );
        let state = clear_trades(state);
        let state = clear_positions(state);
        let state = buy(state, instrument.clone(), Quantity(1000));
        let pos_opt = get_position(&state, &instrument);
        let pos = pos_opt.unwrap();
        assert_eq!(pos.last_qty, Quantity(1000));
        assert_eq!(pos.last_px, Price(dec!(1.1005)));

        let trades = get_trades(&state);
        assert_eq!(trades.len(), 1);
        assert_eq!(trades[0].instrument, instrument);
    }

    #[test]
    fn test_buy_with_orders() {
        let state = State::new();
        let instrument = Instrument::new("EURUSD");
        let state = set_market(
            state,
            instrument.clone(),
            Price(dec!(1.1000)),
            Price(dec!(1.1005)),
        );
        let state = clear_trades(state);
        let state = clear_positions(state);
        let state = clear_open_orders(state);
        let state = buy_with_orders(
            state,
            instrument.clone(),
            Quantity(1000),
            Price(dec!(1.1100)),
            Price(dec!(1.0900)),
        );

        let pos_opt = get_position(&state, &instrument);
        let _ = pos_opt.unwrap();

        let orders = filter_open_orders(&state, |_| true);
        assert_eq!(orders.len(), 2);
        assert_eq!(orders[0].oco_with.len(), 2);
    }

    #[test]
    fn test_sell() {
        let state = State::new();
        let instrument = Instrument::new("EURUSD");
        let state = set_market(
            state,
            instrument.clone(),
            Price(dec!(1.1000)),
            Price(dec!(1.1005)),
        );
        let state = sell(state, instrument.clone(), Quantity(1000));
        let pos = get_position(&state, &instrument).unwrap();
        assert_eq!(pos.last_qty, Quantity(-1000));
        assert_eq!(pos.last_px, Price(dec!(1.1000)));
    }

    #[test]
    fn test_quantity_neg() {
        let q = Quantity(100);
        assert_eq!(-q, Quantity(-100));
        assert_eq!(-(-q), Quantity(100));
    }
}
