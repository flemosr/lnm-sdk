use crate::shared::rest::lnm::base::RestPath;

#[derive(Clone)]
pub(in crate::rest::v3) enum RestPathV3 {
    UtilitiesPing,
    UtilitiesTime,
    FuturesIsolatedTrade,
    FuturesIsolatedTradeAddMargin,
    FuturesIsolatedTradeCancel,
    FuturesIsolatedTradeCashIn,
    FuturesIsolatedTradeClose,
    FuturesIsolatedTradeTakeprofit,
    FuturesIsolatedTradeStoploss,
    FuturesIsolatedTradesCancelAll,
    FuturesIsolatedTradesOpen,
    FuturesIsolatedTradesRunning,
    FuturesIsolatedTradesClosed,
    FuturesIsolatedTradesCanceled,
    FuturesIsolatedFundingFees,
    FuturesCrossOrder,
    FuturesCrossOrderCancel,
    FuturesCrossOrdersCancelAll,
    FuturesCrossOrdersOpen,
    FuturesCrossOrdersFilled,
    FuturesCrossPosition,
    FuturesCrossPositionClose,
    FuturesCrossPositionSetLeverage,
    FuturesCrossDeposit,
    FuturesCrossWithdraw,
    FuturesCrossGetTransfers,
    FuturesCrossFundingFees,
    FuturesDataFundingSettlements,
    FuturesDataTicker,
    FuturesDataGetCandles,
    Account,
    OracleIndex,
    OracleLastPrice,
}

impl RestPath for RestPathV3 {
    fn to_path_string(self) -> String {
        match self {
            RestPathV3::UtilitiesPing => "/ping".into(),
            RestPathV3::UtilitiesTime => "/time".into(),
            RestPathV3::FuturesIsolatedTrade => "/futures/isolated/trade".into(),
            RestPathV3::FuturesIsolatedTradeAddMargin => {
                "/futures/isolated/trade/add-margin".into()
            }
            RestPathV3::FuturesIsolatedTradeCancel => "/futures/isolated/trade/cancel".into(),
            RestPathV3::FuturesIsolatedTradeCashIn => "/futures/isolated/trade/cash-in".into(),
            RestPathV3::FuturesIsolatedTradeClose => "/futures/isolated/trade/close".into(),
            RestPathV3::FuturesIsolatedTradeTakeprofit => {
                "/futures/isolated/trade/takeprofit".into()
            }
            RestPathV3::FuturesIsolatedTradeStoploss => "/futures/isolated/trade/stoploss".into(),
            RestPathV3::FuturesIsolatedTradesCancelAll => {
                "/futures/isolated/trades/cancel-all".into()
            }
            RestPathV3::FuturesIsolatedTradesOpen => "/futures/isolated/trades/open".into(),
            RestPathV3::FuturesIsolatedTradesRunning => "/futures/isolated/trades/running".into(),
            RestPathV3::FuturesIsolatedTradesClosed => "/futures/isolated/trades/closed".into(),
            RestPathV3::FuturesIsolatedTradesCanceled => "/futures/isolated/trades/canceled".into(),
            RestPathV3::FuturesIsolatedFundingFees => "/futures/isolated/funding-fees".into(),
            RestPathV3::FuturesCrossOrder => "/futures/cross/order".into(),
            RestPathV3::FuturesCrossOrderCancel => "/futures/cross/order/cancel".into(),
            RestPathV3::FuturesCrossOrdersCancelAll => "/futures/cross/orders/cancel-all".into(),
            RestPathV3::FuturesCrossOrdersOpen => "/futures/cross/orders/open".into(),
            RestPathV3::FuturesCrossOrdersFilled => "/futures/cross/orders/filled".into(),
            RestPathV3::FuturesCrossPosition => "/futures/cross/position".into(),
            RestPathV3::FuturesCrossPositionClose => "/futures/cross/position/close".into(),
            RestPathV3::FuturesCrossPositionSetLeverage => "/futures/cross/leverage".into(),
            RestPathV3::FuturesCrossDeposit => "/futures/cross/deposit".into(),
            RestPathV3::FuturesCrossWithdraw => "/futures/cross/withdraw".into(),
            RestPathV3::FuturesCrossGetTransfers => "/futures/cross/transfers".into(),
            RestPathV3::FuturesCrossFundingFees => "/futures/cross/funding-fees".into(),
            RestPathV3::FuturesDataFundingSettlements => "/futures/funding-settlements".into(),
            RestPathV3::FuturesDataTicker => "/futures/ticker".into(),
            RestPathV3::FuturesDataGetCandles => "/futures/candles".into(),
            RestPathV3::Account => "/account".into(),
            RestPathV3::OracleIndex => "/oracle/index".into(),
            RestPathV3::OracleLastPrice => "/oracle/last-price".into(),
        }
    }
}
