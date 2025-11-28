use crate::{
    ws::message_types::{AllMids, Bbo, Candle, L2Book, OrderUpdates, Trades, User},
    ActiveAssetCtx, ClearinghouseState, Notification, UserFills, UserFundings,
    UserNonFundingLedgerUpdates, WebData2,
};
use serde::{Deserialize, Serialize};

use ethers::types::H160;

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
#[serde(rename_all = "camelCase")]
pub enum Subscription {
    AllMids,
    Notification { user: H160 },
    WebData2 { user: H160, dex: Option<String> },
    ClearinghouseState { user: H160, dex: Option<String> },
    Candle { coin: String, interval: String },
    L2Book { coin: String },
    Bbo { coin: String },
    Trades { coin: String },
    OrderUpdates { user: H160 },
    UserEvents { user: H160 },
    UserFills { user: H160 },
    UserFundings { user: H160 },
    UserNonFundingLedgerUpdates { user: H160 },
    ActiveAssetCtx { coin: String },
}

#[derive(Deserialize, Clone, Debug)]
#[serde(tag = "channel")]
#[serde(rename_all = "camelCase")]
pub enum Message {
    NoData,
    HyperliquidError(String),
    AllMids(AllMids),
    Trades(Trades),
    L2Book(L2Book),
    Bbo(Bbo),
    User(User),
    UserFills(UserFills),
    Candle(Candle),
    ClearinghouseState(ClearinghouseState),
    SubscriptionResponse,
    OrderUpdates(OrderUpdates),
    UserFundings(UserFundings),
    UserNonFundingLedgerUpdates(UserNonFundingLedgerUpdates),
    Notification(Notification),
    WebData2(WebData2),
    ActiveAssetCtx(ActiveAssetCtx),
    Pong,
}
