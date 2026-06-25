use chrono::{DateTime, Utc};
use serde::Deserialize;

use crate::shared::models::{
    serde_util,
    trade::{TradeExecutionType, TradeSide},
};

/// Inverse futures isolated-margin trade event notification payload.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StreamIsolatedTradeEvent {
    pair: String,
    event: String,
    trade: StreamIsolatedTrade,
}

impl StreamIsolatedTradeEvent {
    pub fn pair(&self) -> &str {
        &self.pair
    }

    pub fn event(&self) -> &str {
        &self.event
    }

    pub fn trade(&self) -> &StreamIsolatedTrade {
        &self.trade
    }
}

/// Inverse futures isolated-margin trade payload fragment.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StreamIsolatedTrade {
    id: Option<String>,
    side: Option<TradeSide>,
    #[serde(rename = "type")]
    trade_type: Option<TradeExecutionType>,
    quantity: Option<f64>,
    margin: Option<f64>,
    leverage: Option<f64>,
    price: Option<f64>,
    opening_fee: Option<f64>,
    #[serde(
        default,
        deserialize_with = "serde_util::datetime_option_rfc3339_or_millis::deserialize"
    )]
    created_at: Option<DateTime<Utc>>,
    #[serde(default)]
    client_id: Option<String>,
}

impl StreamIsolatedTrade {
    pub fn id(&self) -> Option<&str> {
        self.id.as_deref()
    }

    pub fn side(&self) -> Option<TradeSide> {
        self.side
    }

    pub fn trade_type(&self) -> Option<TradeExecutionType> {
        self.trade_type
    }

    pub fn quantity(&self) -> Option<f64> {
        self.quantity
    }

    pub fn margin(&self) -> Option<f64> {
        self.margin
    }

    pub fn leverage(&self) -> Option<f64> {
        self.leverage
    }

    pub fn price(&self) -> Option<f64> {
        self.price
    }

    pub fn opening_fee(&self) -> Option<f64> {
        self.opening_fee
    }

    pub fn created_at(&self) -> Option<DateTime<Utc>> {
        self.created_at
    }

    pub fn client_id(&self) -> Option<&str> {
        self.client_id.as_deref()
    }
}

/// Inverse futures cross-margin order event notification payload.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StreamCrossOrderEvent {
    pair: String,
    event: String,
    order: StreamCrossOrder,
}

impl StreamCrossOrderEvent {
    pub fn pair(&self) -> &str {
        &self.pair
    }

    pub fn event(&self) -> &str {
        &self.event
    }

    pub fn order(&self) -> &StreamCrossOrder {
        &self.order
    }
}

/// Inverse futures cross-margin order payload fragment.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StreamCrossOrder {
    id: Option<String>,
    side: Option<TradeSide>,
    #[serde(rename = "type")]
    order_type: Option<TradeExecutionType>,
    quantity: Option<f64>,
    price: Option<f64>,
    trading_fee: Option<f64>,
    #[serde(default)]
    client_id: Option<String>,
    #[serde(
        default,
        deserialize_with = "serde_util::datetime_option_rfc3339_or_millis::deserialize"
    )]
    created_at: Option<DateTime<Utc>>,
}

impl StreamCrossOrder {
    pub fn id(&self) -> Option<&str> {
        self.id.as_deref()
    }

    pub fn side(&self) -> Option<TradeSide> {
        self.side
    }

    pub fn order_type(&self) -> Option<TradeExecutionType> {
        self.order_type
    }

    pub fn quantity(&self) -> Option<f64> {
        self.quantity
    }

    pub fn price(&self) -> Option<f64> {
        self.price
    }

    pub fn trading_fee(&self) -> Option<f64> {
        self.trading_fee
    }

    pub fn client_id(&self) -> Option<&str> {
        self.client_id.as_deref()
    }

    pub fn created_at(&self) -> Option<DateTime<Utc>> {
        self.created_at
    }
}

/// Inverse futures cross-margin position event notification payload.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StreamCrossPositionEvent {
    pair: String,
    event: String,
    position: StreamCrossPosition,
}

impl StreamCrossPositionEvent {
    pub fn pair(&self) -> &str {
        &self.pair
    }

    pub fn event(&self) -> &str {
        &self.event
    }

    pub fn position(&self) -> &StreamCrossPosition {
        &self.position
    }
}

/// Inverse futures cross-margin position payload fragment.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StreamCrossPosition {
    quantity: Option<f64>,
    leverage: Option<f64>,
    margin: Option<f64>,
    entry_price: Option<f64>,
    liquidation: Option<f64>,
    total_pl: Option<f64>,
    funding_fees: Option<f64>,
    trading_fees: Option<f64>,
    initial_margin: Option<f64>,
    maintenance_margin: Option<f64>,
    running_margin: Option<f64>,
    delta_pl: Option<f64>,
    #[serde(
        default,
        deserialize_with = "serde_util::datetime_option_rfc3339_or_millis::deserialize"
    )]
    updated_at: Option<DateTime<Utc>>,
}

impl StreamCrossPosition {
    pub fn quantity(&self) -> Option<f64> {
        self.quantity
    }

    pub fn leverage(&self) -> Option<f64> {
        self.leverage
    }

    pub fn margin(&self) -> Option<f64> {
        self.margin
    }

    pub fn entry_price(&self) -> Option<f64> {
        self.entry_price
    }

    pub fn liquidation(&self) -> Option<f64> {
        self.liquidation
    }

    pub fn total_pl(&self) -> Option<f64> {
        self.total_pl
    }

    pub fn funding_fees(&self) -> Option<f64> {
        self.funding_fees
    }

    pub fn trading_fees(&self) -> Option<f64> {
        self.trading_fees
    }

    pub fn initial_margin(&self) -> Option<f64> {
        self.initial_margin
    }

    pub fn maintenance_margin(&self) -> Option<f64> {
        self.maintenance_margin
    }

    pub fn running_margin(&self) -> Option<f64> {
        self.running_margin
    }

    pub fn delta_pl(&self) -> Option<f64> {
        self.delta_pl
    }

    pub fn updated_at(&self) -> Option<DateTime<Utc>> {
        self.updated_at
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shared::models::trade::{TradeExecutionType, TradeSide};

    #[test]
    fn stream_isolated_trade_event_deserializes() {
        let event: StreamIsolatedTradeEvent = serde_json::from_str(
            r#"{ "pair": "btc_usd", "event": "open", "trade": { "id": "trade-1", "side": "buy", "type": "limit", "quantity": 1, "margin": 2, "leverage": 3, "price": 4, "openingFee": 5, "createdAt": 0, "clientId": "client-1" } }"#,
        )
        .expect("must deserialize isolated trade event");

        assert_eq!(event.pair(), "btc_usd");
        assert_eq!(event.event(), "open");
        assert_eq!(event.trade().id(), Some("trade-1"));
        assert_eq!(event.trade().side(), Some(TradeSide::Buy));
        assert_eq!(event.trade().trade_type(), Some(TradeExecutionType::Limit));
        assert_eq!(event.trade().quantity(), Some(1.0));
        assert_eq!(event.trade().margin(), Some(2.0));
        assert_eq!(event.trade().leverage(), Some(3.0));
        assert_eq!(event.trade().price(), Some(4.0));
        assert_eq!(event.trade().opening_fee(), Some(5.0));
        assert_eq!(event.trade().created_at().unwrap().timestamp_millis(), 0);
        assert_eq!(event.trade().client_id(), Some("client-1"));
    }

    #[test]
    fn stream_cross_order_event_deserializes() {
        let event: StreamCrossOrderEvent = serde_json::from_str(
            r#"{ "pair": "btc_usd", "event": "new", "order": { "id": "order-1", "side": "buy", "type": "limit", "quantity": 1, "price": 2, "tradingFee": 3, "clientId": "client-1", "createdAt": 0 } }"#,
        )
        .expect("must deserialize cross order event");

        assert_eq!(event.pair(), "btc_usd");
        assert_eq!(event.event(), "new");
        assert_eq!(event.order().id(), Some("order-1"));
        assert_eq!(event.order().side(), Some(TradeSide::Buy));
        assert_eq!(event.order().order_type(), Some(TradeExecutionType::Limit));
        assert_eq!(event.order().quantity(), Some(1.0));
        assert_eq!(event.order().price(), Some(2.0));
        assert_eq!(event.order().trading_fee(), Some(3.0));
        assert_eq!(event.order().client_id(), Some("client-1"));
        assert_eq!(event.order().created_at().unwrap().timestamp_millis(), 0);
    }

    #[test]
    fn stream_cross_position_event_deserializes() {
        let event: StreamCrossPositionEvent = serde_json::from_str(
            r#"{ "pair": "btc_usd", "event": "new", "position": { "quantity": 1, "leverage": 2, "margin": 3, "entryPrice": 4, "liquidation": 5, "totalPl": 6, "fundingFees": 7, "tradingFees": 8, "initialMargin": 9, "maintenanceMargin": 10, "runningMargin": 11, "deltaPl": 12, "updatedAt": 0 } }"#,
        )
        .expect("must deserialize cross position event");

        assert_eq!(event.pair(), "btc_usd");
        assert_eq!(event.event(), "new");
        assert_eq!(event.position().quantity(), Some(1.0));
        assert_eq!(event.position().leverage(), Some(2.0));
        assert_eq!(event.position().margin(), Some(3.0));
        assert_eq!(event.position().entry_price(), Some(4.0));
        assert_eq!(event.position().liquidation(), Some(5.0));
        assert_eq!(event.position().total_pl(), Some(6.0));
        assert_eq!(event.position().funding_fees(), Some(7.0));
        assert_eq!(event.position().trading_fees(), Some(8.0));
        assert_eq!(event.position().initial_margin(), Some(9.0));
        assert_eq!(event.position().maintenance_margin(), Some(10.0));
        assert_eq!(event.position().running_margin(), Some(11.0));
        assert_eq!(event.position().delta_pl(), Some(12.0));
        assert_eq!(event.position().updated_at().unwrap().timestamp_millis(), 0);
    }
}
