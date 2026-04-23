use crate::exchange::{cancel::CancelRequest, modify::ModifyRequest, order::OrderRequest};
pub(crate) use ethers::{
    abi::{encode, ParamType, Tokenizable},
    types::{
        transaction::{
            eip712,
            eip712::{encode_eip712_type, EIP712Domain, Eip712, Eip712Error},
        },
        H160, U256,
    },
    utils::keccak256,
};
use serde::ser::SerializeMap;
use serde::{de::Error as _, Deserialize, Deserializer, Serialize, Serializer};

use super::{cancel::CancelRequestCloid, BuilderInfo};

pub(crate) const HYPERLIQUID_EIP_PREFIX: &str = "HyperliquidTransaction:";

fn eip_712_domain(chain_id: U256) -> EIP712Domain {
    EIP712Domain {
        name: Some("HyperliquidSignTransaction".to_string()),
        version: Some("1".to_string()),
        chain_id: Some(chain_id),
        verifying_contract: Some(
            "0x0000000000000000000000000000000000000000"
                .parse()
                .unwrap(),
        ),
        salt: None,
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UsdSend {
    pub signature_chain_id: U256,
    pub hyperliquid_chain: String,
    pub destination: String,
    pub amount: String,
    pub time: u64,
}

impl Eip712 for UsdSend {
    type Error = Eip712Error;

    fn domain(&self) -> Result<EIP712Domain, Self::Error> {
        Ok(eip_712_domain(self.signature_chain_id))
    }

    fn type_hash() -> Result<[u8; 32], Self::Error> {
        Ok(eip712::make_type_hash(
            format!("{HYPERLIQUID_EIP_PREFIX}UsdSend"),
            &[
                ("hyperliquidChain".to_string(), ParamType::String),
                ("destination".to_string(), ParamType::String),
                ("amount".to_string(), ParamType::String),
                ("time".to_string(), ParamType::Uint(64)),
            ],
        ))
    }

    fn struct_hash(&self) -> Result<[u8; 32], Self::Error> {
        let Self {
            signature_chain_id: _,
            hyperliquid_chain,
            destination,
            amount,
            time,
        } = self;
        let items = vec![
            ethers::abi::Token::Uint(Self::type_hash()?.into()),
            encode_eip712_type(hyperliquid_chain.clone().into_token()),
            encode_eip712_type(destination.clone().into_token()),
            encode_eip712_type(amount.clone().into_token()),
            encode_eip712_type(time.into_token()),
        ];
        Ok(keccak256(encode(&items)))
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UpdateLeverage {
    pub asset: u32,
    pub is_cross: bool,
    pub leverage: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UpdateIsolatedMargin {
    pub asset: u32,
    pub is_buy: bool,
    pub ntli: i64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TopUpIsolatedOnlyMargin {
    pub asset: u32,
    pub leverage: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BulkOrder {
    pub orders: Vec<OrderRequest>,
    #[serde(default)]
    pub grouping: OrderGrouping,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub builder: Option<BuilderInfo>,
}

/// Grouping type for batch orders.
///
/// Serializes as a plain string (`"na"`, `"normalTpsl"`, `"positionTpsl"`) or as an
/// object with a priority rate: `{"p": N}` where N is in units of 1/10_000_000 of
/// filled notional (max 8 bps → `80_000`). When using [`OrderGrouping::PriorityRate`]
/// all orders in the batch must be IOC.
///
/// See <https://hyperliquid.gitbook.io/hyperliquid-docs/for-developers/api/priority-fees#order-write-priority>.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum OrderGrouping {
    Na,
    NormalTpsl,
    PositionTpsl,
    /// Pay a priority tip burned at fill time for faster matching.
    /// Units are 1/10_000_000 of filled notional; max `80_000` (8 bps).
    PriorityRate(u32),
}

impl Default for OrderGrouping {
    fn default() -> Self {
        Self::Na
    }
}

impl Serialize for OrderGrouping {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Na => s.serialize_str("na"),
            Self::NormalTpsl => s.serialize_str("normalTpsl"),
            Self::PositionTpsl => s.serialize_str("positionTpsl"),
            Self::PriorityRate(p) => {
                let mut map = s.serialize_map(Some(1))?;
                map.serialize_entry("p", p)?;
                map.end()
            }
        }
    }
}

impl<'de> Deserialize<'de> for OrderGrouping {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum Raw {
            Str(String),
            Obj { p: u32 },
        }
        match Raw::deserialize(d)? {
            Raw::Str(s) => match s.as_str() {
                "na" => Ok(Self::Na),
                "normalTpsl" => Ok(Self::NormalTpsl),
                "positionTpsl" => Ok(Self::PositionTpsl),
                other => Err(D::Error::custom(format!(
                    "unknown grouping variant: {other}"
                ))),
            },
            Raw::Obj { p } => Ok(Self::PriorityRate(p)),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BulkCancel {
    pub cancels: Vec<CancelRequest>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BulkModify {
    pub modifies: Vec<ModifyRequest>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BulkCancelCloid {
    pub cancels: Vec<CancelRequestCloid>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ApproveAgent {
    pub signature_chain_id: U256,
    pub hyperliquid_chain: String,
    pub agent_address: H160,
    pub agent_name: Option<String>,
    pub nonce: u64,
}

impl Eip712 for ApproveAgent {
    type Error = Eip712Error;

    fn domain(&self) -> Result<EIP712Domain, Self::Error> {
        Ok(eip_712_domain(self.signature_chain_id))
    }

    fn type_hash() -> Result<[u8; 32], Self::Error> {
        Ok(eip712::make_type_hash(
            format!("{HYPERLIQUID_EIP_PREFIX}ApproveAgent"),
            &[
                ("hyperliquidChain".to_string(), ParamType::String),
                ("agentAddress".to_string(), ParamType::Address),
                ("agentName".to_string(), ParamType::String),
                ("nonce".to_string(), ParamType::Uint(64)),
            ],
        ))
    }

    fn struct_hash(&self) -> Result<[u8; 32], Self::Error> {
        let Self {
            signature_chain_id: _,
            hyperliquid_chain,
            agent_address,
            agent_name,
            nonce,
        } = self;
        let items = vec![
            ethers::abi::Token::Uint(Self::type_hash()?.into()),
            encode_eip712_type(hyperliquid_chain.clone().into_token()),
            encode_eip712_type(agent_address.into_token()),
            encode_eip712_type(agent_name.clone().unwrap_or_default().into_token()),
            encode_eip712_type(nonce.into_token()),
        ];
        Ok(keccak256(encode(&items)))
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Withdraw3 {
    pub hyperliquid_chain: String,
    pub signature_chain_id: U256,
    pub amount: String,
    pub time: u64,
    pub destination: String,
}

impl Eip712 for Withdraw3 {
    type Error = Eip712Error;

    fn domain(&self) -> Result<EIP712Domain, Self::Error> {
        Ok(eip_712_domain(self.signature_chain_id))
    }

    fn type_hash() -> Result<[u8; 32], Self::Error> {
        Ok(eip712::make_type_hash(
            format!("{HYPERLIQUID_EIP_PREFIX}Withdraw"),
            &[
                ("hyperliquidChain".to_string(), ParamType::String),
                ("destination".to_string(), ParamType::String),
                ("amount".to_string(), ParamType::String),
                ("time".to_string(), ParamType::Uint(64)),
            ],
        ))
    }

    fn struct_hash(&self) -> Result<[u8; 32], Self::Error> {
        let Self {
            signature_chain_id: _,
            hyperliquid_chain,
            amount,
            time,
            destination,
        } = self;
        let items = vec![
            ethers::abi::Token::Uint(Self::type_hash()?.into()),
            encode_eip712_type(hyperliquid_chain.clone().into_token()),
            encode_eip712_type(destination.clone().into_token()),
            encode_eip712_type(amount.clone().into_token()),
            encode_eip712_type(time.into_token()),
        ];
        Ok(keccak256(encode(&items)))
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SpotSend {
    pub hyperliquid_chain: String,
    pub signature_chain_id: U256,
    pub destination: String,
    pub token: String,
    pub amount: String,
    pub time: u64,
}

impl Eip712 for SpotSend {
    type Error = Eip712Error;

    fn domain(&self) -> Result<EIP712Domain, Self::Error> {
        Ok(eip_712_domain(self.signature_chain_id))
    }

    fn type_hash() -> Result<[u8; 32], Self::Error> {
        Ok(eip712::make_type_hash(
            format!("{HYPERLIQUID_EIP_PREFIX}SpotSend"),
            &[
                ("hyperliquidChain".to_string(), ParamType::String),
                ("destination".to_string(), ParamType::String),
                ("token".to_string(), ParamType::String),
                ("amount".to_string(), ParamType::String),
                ("time".to_string(), ParamType::Uint(64)),
            ],
        ))
    }

    fn struct_hash(&self) -> Result<[u8; 32], Self::Error> {
        let Self {
            signature_chain_id: _,
            hyperliquid_chain,
            destination,
            token,
            amount,
            time,
        } = self;
        let items = vec![
            ethers::abi::Token::Uint(Self::type_hash()?.into()),
            encode_eip712_type(hyperliquid_chain.clone().into_token()),
            encode_eip712_type(destination.clone().into_token()),
            encode_eip712_type(token.clone().into_token()),
            encode_eip712_type(amount.clone().into_token()),
            encode_eip712_type(time.into_token()),
        ];
        Ok(keccak256(encode(&items)))
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SpotUser {
    pub class_transfer: ClassTransfer,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ClassTransfer {
    pub usdc: u64,
    pub to_perp: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VaultTransfer {
    pub vault_address: H160,
    pub is_deposit: bool,
    pub usd: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SetReferrer {
    pub code: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ApproveBuilderFee {
    pub max_fee_rate: String,
    pub builder: String,
    pub nonce: u64,
    pub signature_chain_id: U256,
    pub hyperliquid_chain: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ReserveRequestWeight {
    pub weight: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::exchange::order::{Limit, OrderRequest};

    #[test]
    fn order_grouping_na_serialize() {
        assert_eq!(
            serde_json::to_string(&OrderGrouping::Na).unwrap(),
            r#""na""#
        );
    }

    #[test]
    fn order_grouping_normal_tpsl_serialize() {
        assert_eq!(
            serde_json::to_string(&OrderGrouping::NormalTpsl).unwrap(),
            r#""normalTpsl""#
        );
    }

    #[test]
    fn order_grouping_position_tpsl_serialize() {
        assert_eq!(
            serde_json::to_string(&OrderGrouping::PositionTpsl).unwrap(),
            r#""positionTpsl""#
        );
    }

    #[test]
    fn order_grouping_priority_rate_serialize() {
        let json = serde_json::to_string(&OrderGrouping::PriorityRate(80_000)).unwrap();
        assert_eq!(json, r#"{"p":80000}"#);
    }

    #[test]
    fn order_grouping_deserialize_all_variants() {
        assert!(matches!(
            serde_json::from_str::<OrderGrouping>(r#""na""#).unwrap(),
            OrderGrouping::Na
        ));
        assert!(matches!(
            serde_json::from_str::<OrderGrouping>(r#""normalTpsl""#).unwrap(),
            OrderGrouping::NormalTpsl
        ));
        assert!(matches!(
            serde_json::from_str::<OrderGrouping>(r#""positionTpsl""#).unwrap(),
            OrderGrouping::PositionTpsl
        ));
        assert!(matches!(
            serde_json::from_str::<OrderGrouping>(r#"{"p":80000}"#).unwrap(),
            OrderGrouping::PriorityRate(80_000)
        ));
    }

    #[test]
    fn order_grouping_deserialize_unknown_string_errors() {
        assert!(serde_json::from_str::<OrderGrouping>(r#""bogus""#).is_err());
    }

    #[test]
    fn batch_order_with_priority_rate_roundtrip() {
        let batch = BulkOrder {
            orders: vec![OrderRequest {
                asset: 0,
                is_buy: true,
                limit_px: "50000".to_string(),
                sz: "0.1".to_string(),
                reduce_only: false,
                order_type: crate::exchange::order::Order::Limit(Limit {
                    tif: "Ioc".to_string(),
                }),
                cloid: None,
            }],
            grouping: OrderGrouping::PriorityRate(80_000),
            builder: None,
        };

        let json = serde_json::to_string(&batch).unwrap();
        assert!(
            json.contains(r#""grouping":{"p":80000}"#),
            "json should embed priority rate object, got: {json}"
        );

        let parsed: BulkOrder = serde_json::from_str(&json).unwrap();
        assert!(matches!(
            parsed.grouping,
            OrderGrouping::PriorityRate(80_000)
        ));
    }
}
