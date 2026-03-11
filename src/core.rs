use std::collections::HashMap;
use uuid::Uuid;

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
    pub bid: f64,
    pub ask: f64,
}

#[derive(Debug, Clone)]
pub struct Position {
    pub instrument: String,
    pub last_qty: f64,
    pub last_px: f64,
}

#[derive(Debug, Clone)]
pub struct Order {
    pub id: Uuid,
    pub side: Side,
    pub qty: f64,
    pub instrument: String,
    pub order_type: OrderType,
    pub px: f64,
    pub oco_with: Vec<Uuid>,
}

#[derive(Debug, Clone)]
pub struct Trade {
    pub instrument: String,
    pub last_qty: f64,
    pub last_px: f64,
}

#[derive(Debug, Default, Clone)]
pub struct State {
    pub trades: Vec<Trade>,
    pub positions: HashMap<String, Position>,
    pub market: HashMap<String, MarketData>,
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
pub fn get_market(state: &State, cross: &str) -> MarketData {
    state
        .market
        .get(cross)
        .cloned()
        .expect(&format!("Market not found for cross: {}", cross))
}

#[allow(dead_code)]
pub fn set_market(mut state: State, cross: &str, bid: f64, ask: f64) -> State {
    state.market.insert(cross.to_string(), MarketData { bid, ask });
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
pub fn get_positions(state: &State) -> HashMap<String, Position> {
    state.positions.clone()
}

#[allow(dead_code)]
pub fn get_position(state: &State, instrument: &str) -> Option<Position> {
    state.positions.get(instrument).cloned()
}

#[allow(dead_code)]
pub fn set_position(mut state: State, instrument: &str, qty: f64, price: f64) -> State {
    state.positions.insert(
        instrument.to_string(),
        make_position(instrument, qty, price),
    );
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

pub fn register_trade(mut state: State, instrument: &str, qty: f64, price: f64) -> State {
    state.trades.push(make_trade(instrument, qty, price));
    state
}

fn trade(mut state: State, cross: &str, qty: f64, price: f64) -> State {
    state.trades.push(make_trade(cross, qty, price));
    state
        .positions
        .insert(cross.to_string(), make_position(cross, qty, price));
    state
}

pub fn buy(state: State, cross: &str, qty: f64) -> State {
    let market_data = get_market(&state, cross);
    let price = market_data.ask;
    trade(state, cross, qty, price)
}

pub fn buy_with_orders(state: State, cross: &str, qty: f64, target: f64, stop: f64) -> State {
    let take_profit = create_order(Side::Sell, qty, cross, OrderType::Limit, target);
    let stop_loss = create_order(Side::Sell, qty, cross, OrderType::Stop, stop);

    let market_data = get_market(&state, cross);
    let price = market_data.ask;

    let state = trade(state, cross, qty, price);

    let oco_orders = make_oco(vec![take_profit, stop_loss]);
    submit_orders(state, oco_orders)
}

pub fn sell(state: State, cross: &str, qty: f64) -> State {
    let market_data = get_market(&state, cross);
    let price = market_data.bid;
    trade(state, cross, -qty, price)
}

// ----------------------------------------------------------------
// Helper functions
// ----------------------------------------------------------------

pub fn make_position(instrument: &str, qty: f64, price: f64) -> Position {
    Position {
        instrument: instrument.to_string(),
        last_qty: qty,
        last_px: price,
    }
}

pub fn create_order(
    side: Side,
    qty: f64,
    instrument: &str,
    order_type: OrderType,
    price: f64,
) -> Order {
    Order {
        id: Uuid::new_v4(),
        side,
        qty,
        instrument: instrument.to_string(),
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

pub fn make_trade(instrument: &str, qty: f64, price: f64) -> Trade {
    Trade {
        instrument: instrument.to_string(),
        last_qty: qty,
        last_px: price,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_market() {
        let state = State::new();
        let state = set_market(state, "EURUSD", 1.1000, 1.1005);
        let m = get_market(&state, "EURUSD");
        assert_eq!(m.bid, 1.1000);
        assert_eq!(m.ask, 1.1005);
    }

    #[test]
    fn test_buy() {
        let state = State::new();
        let state = set_market(state, "EURUSD", 1.1000, 1.1005);
        let state = clear_trades(state);
        let state = clear_positions(state);
        let state = buy(state, "EURUSD", 1000.0);
        let pos_opt = get_position(&state, "EURUSD");
        let pos = pos_opt.unwrap();
        assert_eq!(pos.last_qty, 1000.0);
        assert_eq!(pos.last_px, 1.1005);

        let trades = get_trades(&state);
        assert_eq!(trades.len(), 1);
        assert_eq!(trades[0].instrument, "EURUSD");
    }

    #[test]
    fn test_buy_with_orders() {
        let state = State::new();
        let state = set_market(state, "EURUSD", 1.1000, 1.1005);
        let state = clear_trades(state);
        let state = clear_positions(state);
        let state = clear_open_orders(state);
        let state = buy_with_orders(state, "EURUSD", 1000.0, 1.1100, 1.0900);

        let pos_opt = get_position(&state, "EURUSD");
        let _ = pos_opt.unwrap();

        let orders = filter_open_orders(&state, |_| true);
        assert_eq!(orders.len(), 2);
        assert_eq!(orders[0].oco_with.len(), 2);
    }
}
