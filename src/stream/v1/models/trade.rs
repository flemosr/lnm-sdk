use chrono::{DateTime, Utc};
use serde::Deserialize;
use uuid::Uuid;

use crate::shared::models::{
    client_id::ClientId,
    cross_leverage::CrossLeverage,
    leverage::Leverage,
    margin::Margin,
    price::Price,
    quantity::OrderQuantity,
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
    id: Option<Uuid>,
    side: Option<TradeSide>,
    #[serde(rename = "type")]
    trade_type: Option<TradeExecutionType>,
    quantity: Option<OrderQuantity>,
    margin: Option<Margin>,
    leverage: Option<Leverage>,
    #[serde(default, deserialize_with = "serde_util::price_option::deserialize")]
    price: Option<Price>,
    opening_fee: Option<u64>,
    #[serde(
        default,
        deserialize_with = "serde_util::datetime_option_rfc3339_or_millis::deserialize"
    )]
    created_at: Option<DateTime<Utc>>,
    #[serde(
        default,
        deserialize_with = "serde_util::client_id_option::deserialize"
    )]
    client_id: Option<ClientId>,
}

impl StreamIsolatedTrade {
    pub fn id(&self) -> Option<Uuid> {
        self.id
    }

    pub fn side(&self) -> Option<TradeSide> {
        self.side
    }

    pub fn trade_type(&self) -> Option<TradeExecutionType> {
        self.trade_type
    }

    pub fn quantity(&self) -> Option<OrderQuantity> {
        self.quantity
    }

    pub fn margin(&self) -> Option<Margin> {
        self.margin
    }

    pub fn leverage(&self) -> Option<Leverage> {
        self.leverage
    }

    pub fn price(&self) -> Option<Price> {
        self.price
    }

    pub fn opening_fee(&self) -> Option<u64> {
        self.opening_fee
    }

    pub fn created_at(&self) -> Option<DateTime<Utc>> {
        self.created_at
    }

    pub fn client_id(&self) -> Option<&ClientId> {
        self.client_id.as_ref()
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
    id: Option<Uuid>,
    side: Option<TradeSide>,
    #[serde(rename = "type")]
    order_type: Option<TradeExecutionType>,
    quantity: Option<OrderQuantity>,
    #[serde(default, deserialize_with = "serde_util::price_option::deserialize")]
    price: Option<Price>,
    trading_fee: Option<u64>,
    #[serde(
        default,
        deserialize_with = "serde_util::client_id_option::deserialize"
    )]
    client_id: Option<ClientId>,
    #[serde(
        default,
        deserialize_with = "serde_util::datetime_option_rfc3339_or_millis::deserialize"
    )]
    created_at: Option<DateTime<Utc>>,
}

impl StreamCrossOrder {
    pub fn id(&self) -> Option<Uuid> {
        self.id
    }

    pub fn side(&self) -> Option<TradeSide> {
        self.side
    }

    pub fn order_type(&self) -> Option<TradeExecutionType> {
        self.order_type
    }

    pub fn quantity(&self) -> Option<OrderQuantity> {
        self.quantity
    }

    pub fn price(&self) -> Option<Price> {
        self.price
    }

    pub fn trading_fee(&self) -> Option<u64> {
        self.trading_fee
    }

    pub fn client_id(&self) -> Option<&ClientId> {
        self.client_id.as_ref()
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
    quantity: Option<i64>,
    leverage: Option<CrossLeverage>,
    margin: Option<u64>,
    #[serde(default, deserialize_with = "serde_util::price_option::deserialize")]
    entry_price: Option<Price>,
    #[serde(default, deserialize_with = "serde_util::price_option::deserialize")]
    liquidation: Option<Price>,
    total_pl: Option<i64>,
    funding_fees: Option<i64>,
    trading_fees: Option<u64>,
    initial_margin: Option<u64>,
    maintenance_margin: Option<u64>,
    running_margin: Option<u64>,
    delta_pl: Option<i64>,
    #[serde(
        default,
        deserialize_with = "serde_util::datetime_option_rfc3339_or_millis::deserialize"
    )]
    updated_at: Option<DateTime<Utc>>,
}

impl StreamCrossPosition {
    pub fn quantity(&self) -> Option<i64> {
        self.quantity
    }

    pub fn leverage(&self) -> Option<CrossLeverage> {
        self.leverage
    }

    pub fn margin(&self) -> Option<u64> {
        self.margin
    }

    pub fn entry_price(&self) -> Option<Price> {
        self.entry_price
    }

    pub fn liquidation(&self) -> Option<Price> {
        self.liquidation
    }

    pub fn total_pl(&self) -> Option<i64> {
        self.total_pl
    }

    pub fn funding_fees(&self) -> Option<i64> {
        self.funding_fees
    }

    pub fn trading_fees(&self) -> Option<u64> {
        self.trading_fees
    }

    pub fn initial_margin(&self) -> Option<u64> {
        self.initial_margin
    }

    pub fn maintenance_margin(&self) -> Option<u64> {
        self.maintenance_margin
    }

    pub fn running_margin(&self) -> Option<u64> {
        self.running_margin
    }

    pub fn delta_pl(&self) -> Option<i64> {
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
        let trade_id =
            Uuid::parse_str("00000000-0000-0000-0000-000000000001").expect("must parse trade id");
        let client_id = ClientId::try_from("client-1").expect("must parse client id");
        let event: StreamIsolatedTradeEvent = serde_json::from_str(
            r#"{ "pair": "btc_usd", "event": "open", "trade": { "id": "00000000-0000-0000-0000-000000000001", "side": "buy", "type": "limit", "quantity": 1, "margin": 2, "leverage": 3, "price": 4, "openingFee": 5, "createdAt": 0, "clientId": "client-1" } }"#,
        )
        .expect("must deserialize isolated trade event");

        assert_eq!(event.pair(), "btc_usd");
        assert_eq!(event.event(), "open");
        assert_eq!(event.trade().id(), Some(trade_id));
        assert_eq!(event.trade().side(), Some(TradeSide::Buy));
        assert_eq!(event.trade().trade_type(), Some(TradeExecutionType::Limit));
        assert_eq!(
            event.trade().quantity(),
            Some(OrderQuantity::try_from(1).unwrap())
        );
        assert_eq!(event.trade().margin(), Some(Margin::try_from(2).unwrap()));
        assert_eq!(
            event.trade().leverage(),
            Some(Leverage::try_from(3.0).unwrap())
        );
        assert_eq!(event.trade().price(), Some(Price::try_from(4.0).unwrap()));
        assert_eq!(event.trade().opening_fee(), Some(5));
        assert_eq!(event.trade().created_at().unwrap().timestamp_millis(), 0);
        assert_eq!(event.trade().client_id(), Some(&client_id));
    }

    #[test]
    fn stream_isolated_trade_event_deserializes_partial_close_event() {
        let trade_id =
            Uuid::parse_str("00000000-0000-0000-0000-000000000003").expect("must parse trade id");
        let client_id = ClientId::try_from("client-3").expect("must parse client id");
        let event: StreamIsolatedTradeEvent = serde_json::from_str(
            r#"{ "pair": "btc_usd", "event": "closed", "trade": { "id": "00000000-0000-0000-0000-000000000003", "clientId": "client-3" } }"#,
        )
        .expect("must deserialize partial isolated trade event");

        assert_eq!(event.trade().id(), Some(trade_id));
        assert_eq!(event.trade().client_id(), Some(&client_id));
        assert_eq!(event.trade().side(), None);
        assert_eq!(event.trade().quantity(), None);
        assert_eq!(event.trade().margin(), None);
        assert_eq!(event.trade().price(), None);
    }

    #[test]
    fn stream_cross_order_event_deserializes() {
        let order_id =
            Uuid::parse_str("00000000-0000-0000-0000-000000000002").expect("must parse order id");
        let client_id = ClientId::try_from("client-1").expect("must parse client id");
        let event: StreamCrossOrderEvent = serde_json::from_str(
            r#"{ "pair": "btc_usd", "event": "new", "order": { "id": "00000000-0000-0000-0000-000000000002", "side": "buy", "type": "limit", "quantity": 1, "price": 2, "tradingFee": 3, "clientId": "client-1", "createdAt": 0 } }"#,
        )
        .expect("must deserialize cross order event");

        assert_eq!(event.pair(), "btc_usd");
        assert_eq!(event.event(), "new");
        assert_eq!(event.order().id(), Some(order_id));
        assert_eq!(event.order().side(), Some(TradeSide::Buy));
        assert_eq!(event.order().order_type(), Some(TradeExecutionType::Limit));
        assert_eq!(
            event.order().quantity(),
            Some(OrderQuantity::try_from(1).unwrap())
        );
        assert_eq!(event.order().price(), Some(Price::try_from(2.0).unwrap()));
        assert_eq!(event.order().trading_fee(), Some(3));
        assert_eq!(event.order().client_id(), Some(&client_id));
        assert_eq!(event.order().created_at().unwrap().timestamp_millis(), 0);
    }

    #[test]
    fn stream_cross_order_event_deserializes_partial_cancel_event() {
        let order_id =
            Uuid::parse_str("00000000-0000-0000-0000-000000000004").expect("must parse order id");
        let event: StreamCrossOrderEvent = serde_json::from_str(
            r#"{ "pair": "btc_usd", "event": "canceled", "order": { "id": "00000000-0000-0000-0000-000000000004", "tradingFee": null, "clientId": "" } }"#,
        )
        .expect("must deserialize partial cross order event");

        assert_eq!(event.order().id(), Some(order_id));
        assert_eq!(event.order().trading_fee(), None);
        assert_eq!(event.order().client_id(), None);
        assert_eq!(event.order().side(), None);
        assert_eq!(event.order().quantity(), None);
        assert_eq!(event.order().price(), None);
    }

    #[test]
    fn stream_cross_position_event_deserializes() {
        let event: StreamCrossPositionEvent = serde_json::from_str(
            r#"{ "pair": "btc_usd", "event": "new", "position": { "quantity": 1, "leverage": 2, "margin": 3, "entryPrice": 4, "liquidation": 5, "totalPl": 6, "fundingFees": 7, "tradingFees": 8, "initialMargin": 9, "maintenanceMargin": 10, "runningMargin": 11, "deltaPl": 12, "updatedAt": 0 } }"#,
        )
        .expect("must deserialize cross position event");

        assert_eq!(event.pair(), "btc_usd");
        assert_eq!(event.event(), "new");
        assert_eq!(event.position().quantity(), Some(1));
        assert_eq!(
            event.position().leverage(),
            Some(CrossLeverage::try_from(2).unwrap())
        );
        assert_eq!(event.position().margin(), Some(3));
        assert_eq!(
            event.position().entry_price(),
            Some(Price::try_from(4.0).unwrap())
        );
        assert_eq!(
            event.position().liquidation(),
            Some(Price::try_from(5.0).unwrap())
        );
        assert_eq!(event.position().total_pl(), Some(6));
        assert_eq!(event.position().funding_fees(), Some(7));
        assert_eq!(event.position().trading_fees(), Some(8));
        assert_eq!(event.position().initial_margin(), Some(9));
        assert_eq!(event.position().maintenance_margin(), Some(10));
        assert_eq!(event.position().running_margin(), Some(11));
        assert_eq!(event.position().delta_pl(), Some(12));
        assert_eq!(event.position().updated_at().unwrap().timestamp_millis(), 0);
    }
}
