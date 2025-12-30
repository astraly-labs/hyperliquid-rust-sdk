use std::collections::HashMap;

use ethers::abi::ethereum_types::H128;
use rust_decimal::Decimal;
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Meta {
    pub universe: Vec<AssetMeta>,
    pub margin_tables: Vec<(u32, MarginTableData)>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct SpotMeta {
    pub universe: Vec<SpotAssetMeta>,
    pub tokens: Vec<TokenInfo>,
}

impl SpotMeta {
    pub fn add_pair_and_name_to_index_map(
        &self,
        mut coin_to_asset: HashMap<String, u32>,
    ) -> HashMap<String, u32> {
        let index_to_name: HashMap<usize, &str> = self
            .tokens
            .iter()
            .map(|info| (info.index, info.name.as_str()))
            .collect();

        for asset in self.universe.iter() {
            let spot_ind: u32 = 10000 + asset.index as u32;
            let name_to_ind = (asset.name.clone(), spot_ind);

            let Some(token_1_name) = index_to_name.get(&asset.tokens[0]) else {
                continue;
            };

            let Some(token_2_name) = index_to_name.get(&asset.tokens[1]) else {
                continue;
            };

            coin_to_asset.insert(format!("{}/{}", token_1_name, token_2_name), spot_ind);
            coin_to_asset.insert(name_to_ind.0, name_to_ind.1);
        }

        coin_to_asset
    }
}

#[derive(Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum SpotMetaAndAssetCtxs {
    SpotMeta(SpotMeta),
    Context(Vec<SpotAssetContext>),
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SpotAssetContext {
    pub day_ntl_vlm: String,
    pub mark_px: String,
    pub mid_px: Option<String>,
    pub prev_day_px: String,
    pub circulating_supply: String,
    pub coin: String,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AssetMeta {
    pub name: String,
    pub sz_decimals: u32,
    pub margin_table_id: u32,
    pub max_leverage: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub only_isolated: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_delisted: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub margin_mode: Option<MarginMode>,
}

#[derive(Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum MarginMode {
    StrictIsolated,
    NoCross,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MarginTableData {
    pub description: String,
    pub margin_tiers: Vec<MarginTier>,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MarginTier {
    pub lower_bound: Decimal,
    pub max_leverage: u32,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SpotAssetMeta {
    pub tokens: [usize; 2],
    pub name: String,
    pub index: usize,
    pub is_canonical: bool,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TokenInfo {
    pub name: String,
    pub sz_decimals: u8,
    pub wei_decimals: u8,
    pub index: usize,
    pub token_id: H128,
    pub is_canonical: bool,
}

#[derive(Deserialize, Debug, Clone)]
pub struct PerpDexMeta {
    pub universe: Vec<AssetMeta>,
}

impl PerpDexMeta {
    /// Creates a mapping of perp names to their asset IDs for builder-deployed perps.
    ///
    /// Builder-deployed perps use the formula: 100000 + perp_dex_index * 10000 + index_in_meta
    ///
    /// # Arguments
    /// * `perp_dex_index` - The index of the perp dex (obtained from perpDexs endpoint)
    /// * `coin_to_asset` - Optional existing HashMap to extend with perp mappings
    ///
    /// # Example
    /// For test:ABC on testnet with perp_dex_index=1 and index_in_meta=0,
    /// the asset ID will be: 100000 + 1*10000 + 0 = 110000
    pub fn add_perp_to_asset_map(
        &self,
        perp_dex_index: usize,
        mut coin_to_asset: HashMap<String, u32>,
    ) -> HashMap<String, u32> {
        for (index_in_meta, asset) in self.universe.iter().enumerate() {
            let asset_id: u32 = 100000 + (perp_dex_index as u32 * 10000) + index_in_meta as u32;
            coin_to_asset.insert(asset.name.clone(), asset_id);
        }

        coin_to_asset
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[ignore = "incomplete JSON payload"]
    #[test]
    fn test_perp_dex_meta_parsing() {
        // Test parsing meta response for a perp dex (flxn example from testnet)
        let json = r#"{
            "universe": [
                {
                    "szDecimals": 2,
                    "name": "flxn:TSLA",
                    "maxLeverage": 10
                }
            ],
            "marginTables": []
        }"#;

        let meta: Meta = serde_json::from_str(json).unwrap();
        assert_eq!(meta.universe.len(), 1);
        assert_eq!(meta.universe[0].name, "flxn:TSLA");
        assert_eq!(meta.universe[0].sz_decimals, 2);
        assert_eq!(meta.universe[0].max_leverage, 10);
    }

    #[ignore = "incomplete JSON payload"]
    #[test]
    fn test_meta_full_parsing() {
        let json = r#"{
            "universe": [
                {
                    "name": "BTC",
                    "szDecimals": 5,
                    "maxLeverage": 50
                },
                {
                    "name": "ETH",
                    "szDecimals": 4,
                    "maxLeverage": 50
                },
                {
                    "name": "HPOS",
                    "szDecimals": 0,
                    "maxLeverage": 3,
                    "onlyIsolated": true
                },
                {
                    "name": "LOOM",
                    "szDecimals": 1,
                    "maxLeverage": 3,
                    "isDelisted": true,
                    "marginMode": "strictIsolated",
                    "onlyIsolated": true
                }
            ],
            "marginTables": [
                [
                    50,
                    {
                        "description": "",
                        "marginTiers": [
                            {
                                "lowerBound": "0.0",
                                "maxLeverage": 50
                            }
                        ]
                    }
                ],
                [
                    51,
                    {
                        "description": "tiered 10x",
                        "marginTiers": [
                            {
                                "lowerBound": "0.0",
                                "maxLeverage": 10
                            },
                            {
                                "lowerBound": "3000000.0",
                                "maxLeverage": 5
                            }
                        ]
                    }
                ]
            ]
        }"#;

        let meta: Meta = serde_json::from_str(json).unwrap();

        assert_eq!(meta.universe.len(), 4);
        assert_eq!(meta.universe[0].name, "BTC");
        assert_eq!(meta.universe[0].max_leverage, 50);
        assert_eq!(meta.universe[0].only_isolated, None);

        assert_eq!(meta.universe[2].name, "HPOS");
        assert_eq!(meta.universe[2].only_isolated, Some(true));

        assert_eq!(meta.universe[3].name, "LOOM");
        assert_eq!(meta.universe[3].is_delisted, Some(true));
        assert_eq!(
            meta.universe[3].margin_mode,
            Some(MarginMode::StrictIsolated)
        );

        assert_eq!(meta.margin_tables.len(), 2);
        assert_eq!(meta.margin_tables[0].0, 50);
        assert_eq!(meta.margin_tables[0].1.description, "");
        assert_eq!(meta.margin_tables[0].1.margin_tiers.len(), 1);
        assert_eq!(
            meta.margin_tables[0].1.margin_tiers[0].lower_bound,
            Decimal::ZERO
        );
        assert_eq!(meta.margin_tables[0].1.margin_tiers[0].max_leverage, 50);

        assert_eq!(meta.margin_tables[1].0, 51);
        assert_eq!(meta.margin_tables[1].1.description, "tiered 10x");
        assert_eq!(meta.margin_tables[1].1.margin_tiers.len(), 2);
        assert_eq!(
            meta.margin_tables[1].1.margin_tiers[1].lower_bound,
            dec!(3000000.0)
        );
        assert_eq!(meta.margin_tables[1].1.margin_tiers[1].max_leverage, 5);
    }

    #[test]
    fn test_perp_dex_meta_asset_id_calculation() {
        // Test the asset ID calculation formula
        // For test:ABC with perp_dex_index=1 and index_in_meta=0, asset should be 110000
        let perp_dex_meta = PerpDexMeta {
            universe: vec![
                AssetMeta {
                    name: "test:ABC".to_string(),
                    sz_decimals: 2,
                    margin_table_id: 0,
                    max_leverage: 10,
                    only_isolated: None,
                    is_delisted: None,
                    margin_mode: None,
                },
                AssetMeta {
                    name: "test:XYZ".to_string(),
                    sz_decimals: 3,
                    margin_table_id: 0,
                    max_leverage: 10,
                    only_isolated: None,
                    is_delisted: None,
                    margin_mode: None,
                },
            ],
        };

        let perp_dex_index = 1;
        let asset_map = perp_dex_meta.add_perp_to_asset_map(perp_dex_index, HashMap::new());

        // test:ABC is at index 0: 100000 + 1*10000 + 0 = 110000
        assert_eq!(asset_map.get("test:ABC"), Some(&110000));

        // test:XYZ is at index 1: 100000 + 1*10000 + 1 = 110001
        assert_eq!(asset_map.get("test:XYZ"), Some(&110001));
    }

    #[test]
    fn test_perp_dex_meta_multiple_indices() {
        // Test with different perp_dex_index values
        let perp_dex_meta = PerpDexMeta {
            universe: vec![AssetMeta {
                name: "xyz:XYZ100".to_string(),
                sz_decimals: 2,
                margin_table_id: 0,
                max_leverage: 10,
                only_isolated: None,
                is_delisted: None,
                margin_mode: None,
            }],
        };

        // Test with perp_dex_index = 0
        let asset_map = perp_dex_meta.add_perp_to_asset_map(0, HashMap::new());
        assert_eq!(asset_map.get("xyz:XYZ100"), Some(&100000));

        // Test with perp_dex_index = 2
        let asset_map = perp_dex_meta.add_perp_to_asset_map(2, HashMap::new());
        assert_eq!(asset_map.get("xyz:XYZ100"), Some(&120000));
    }

    #[test]
    fn test_perp_dex_meta_extend_existing_map() {
        // Test that the method can extend an existing HashMap
        let perp_dex_meta = PerpDexMeta {
            universe: vec![AssetMeta {
                name: "test:ABC".to_string(),
                sz_decimals: 2,
                margin_table_id: 0,
                max_leverage: 10,
                only_isolated: None,
                is_delisted: None,
                margin_mode: None,
            }],
        };

        let mut existing_map = HashMap::new();
        existing_map.insert("existing:COIN".to_string(), 99999);

        let asset_map = perp_dex_meta.add_perp_to_asset_map(1, existing_map);

        // Check that both the existing and new entries are present
        assert_eq!(asset_map.get("existing:COIN"), Some(&99999));
        assert_eq!(asset_map.get("test:ABC"), Some(&110000));
    }
}
