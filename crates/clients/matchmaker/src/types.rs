use std::str::FromStr;

use ethers::types::{Bytes, H256, U64, Address};
use serde::{Deserialize, Serialize, Serializer, Deserializer, ser::SerializeSeq};

/// A bundle of transactions to send to the matchmaker.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BundleRequest {
    /// The version of the MEV-share API to use.
    pub version: ProtocolVersion,
    /// Data used by block builders to check if the bundle should be considered for inclusion.
    pub inclusion: Inclusion,
    /// The transactions to include in the bundle.
    pub body: Vec<BundleTx>,

    #[serde(rename = "validity", skip_serializing_if = "Option::is_none")]
    pub validity: Option<Validity>,
    /// Preferences on what data should be shared about the bundle and its transactions
    #[serde(rename = "privacy", skip_serializing_if = "Option::is_none")]
    pub privacy: Option<Privacy>,

}

/// Data used by block builders to check if the bundle should be considered for inclusion.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Inclusion {
    /// The first block the bundle is valid for.
    pub block: U64,
    /// The last block the bundle is valid for.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_block: Option<U64>,
}

/// A bundle tx, which can either be a transaction hash, or a full tx.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
#[serde(rename_all = "camelCase")]
pub enum BundleTx {
    /// The hash of the transaction we are trying to backrun.
    TxHash {
        /// Tx hash.
        hash: H256,
    },
    /// A new signed transaction.
    #[serde(rename_all = "camelCase")]
    Tx {
        /// Bytes of the signed transaction.
        tx: Bytes,
        /// If true, the transaction can revert without the bundle being considered invalid.
        can_revert: bool,
    },
}

/// Response from the matchmaker after sending a bundle.
#[derive(Deserialize, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SendBundleResponse {
    /// Hash of the bundle bodies.
    bundle_hash: H256,
}

/// The version of the MEV-share API to use.
#[derive(Deserialize, Debug, Serialize, Clone, Default)]
pub enum ProtocolVersion {
    #[default]
    #[serde(rename = "beta-1")]
    /// The beta-1 version of the API.
    Beta1,
    /// The 0.1 version of the API.
    #[serde(rename = "v0.1")]
    V0_1,
}

/// Requirements for the bundle to be included in the block.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Validity {
    /// Specifies the minimum percent of a given bundle's earnings to redistribute
    /// for it to be included in a builder's block.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refund: Option<Vec<Refund>>,
    /// Specifies what addresses should receive what percent of the overall refund for this bundle,
    /// if it is enveloped by another bundle (eg. a searcher backrun).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refund_config: Option<Vec<RefundConfig>>,
}

/// Specifies the minimum percent of a given bundle's earnings to redistribute
/// for it to be included in a builder's block.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Refund {
    /// The index of the transaction in the bundle.
    pub body_idx: u64,
    /// The minimum percent of the bundle's earnings to redistribute.
    pub percent: u64,
}

/// Specifies what addresses should receive what percent of the overall refund for this bundle,
/// if it is enveloped by another bundle (eg. a searcher backrun).
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct RefundConfig {
    /// The address to refund.
    pub address: Address,
    /// The minimum percent of the bundle's earnings to redistribute.
    pub percent: u64,
    
}

/// Preferences on what data should be shared about the bundle and its transactions
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Privacy {
    /// Hints on what data should be shared about the bundle and its transactions
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hints: Option<PrivacyHint>,
    /// The addresses of the builders that should be allowed to see the bundle.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub builders: Option<Vec<Address>>,
}

/// Hints on what data should be shared about the bundle and its transactions
#[derive(Clone, Debug, PartialEq, Default)]
pub struct PrivacyHint {
    /// The calldata of the bundle's transactions should be shared.
    pub calldata: bool,
    /// The address of the bundle's transactions should be shared.
    pub contract_address: bool,
    /// The logs of the bundle's transactions should be shared.
    pub logs: bool,
    /// The function selector of the bundle's transactions should be shared.
    pub function_selector: bool,
    /// The hash of the bundle's transactions should be shared.
    pub hash: bool,
    /// The hash of the bundle should be shared.
    pub tx_hash: bool,
}

#[allow(missing_docs)]
impl PrivacyHint {
    pub fn with_calldata(mut self) -> Self {
        self.calldata = true;
        self
    }

    pub fn with_contract_address(mut self) -> Self {
        self.contract_address = true;
        self
    }

    pub fn with_logs(mut self) -> Self {
        self.logs = true;
        self
    }

    pub fn with_function_selector(mut self) -> Self {
        self.function_selector = true;
        self
    }

    pub fn with_hash(mut self) -> Self {
        self.hash = true;
        self
    }

    pub fn with_tx_hash(mut self) -> Self {
        self.tx_hash = true;
        self
    }

    pub fn has_calldata(&self) -> bool {
        self.calldata
    }

    pub fn has_contract_address(&self) -> bool {
        self.contract_address
    }

    pub fn has_logs(&self) -> bool {
        self.logs
    }

    pub fn has_function_selector(&self) -> bool {
        self.function_selector
    }

    pub fn has_hash(&self) -> bool {
        self.hash
    }

    pub fn has_tx_hash(&self) -> bool {
        self.tx_hash
    }

    fn num_hints(&self) -> usize {
        let mut num_hints = 0;
        if self.calldata {
            num_hints += 1;
        }
        if self.contract_address {
            num_hints += 1;
        }
        if self.logs {
            num_hints += 1;
        }
        if self.function_selector {
            num_hints += 1;
        }
        if self.hash {
            num_hints += 1;
        }
        if self.tx_hash {
            num_hints += 1;
        }
        num_hints
    }
}

impl Serialize for PrivacyHint {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut seq = serializer.serialize_seq(Some(self.num_hints()))?;
        if self.calldata {
            seq.serialize_element("calldata")?;
        }
        if self.contract_address {
            seq.serialize_element("contract_address")?;
        }
        if self.logs {
            seq.serialize_element("logs")?;
        }
        if self.function_selector {
            seq.serialize_element("function_selector")?;
        }
        if self.hash {
            seq.serialize_element("hash")?;
        }
        if self.tx_hash {
            seq.serialize_element("tx_hash")?;
        }
        seq.end()
    }
}

impl<'de> Deserialize<'de> for PrivacyHint {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let hints = Vec::<String>::deserialize(deserializer)?;
        let mut privacy_hint = PrivacyHint::default();
        for hint in hints {
            match hint.as_str() {
                "calldata" => privacy_hint.calldata = true,
                "contract_address" => privacy_hint.contract_address = true,
                "logs" => privacy_hint.logs = true,
                "function_selector" => privacy_hint.function_selector = true,
                "hash" => privacy_hint.hash = true,
                "tx_hash" => privacy_hint.tx_hash = true,
                _ => return Err(serde::de::Error::custom("invalid privacy hint")),
            }
        }
        Ok(privacy_hint)
    }
}


impl BundleRequest {
    /// Create a new bundle request.
    pub fn new(
        block_num: U64,
        max_block: Option<U64>,
        version: ProtocolVersion,
        transactions: Vec<BundleTx>,
    ) -> Self {



        Self {
            version,
            inclusion: Inclusion {
                block: block_num,
                max_block,
            },
           body: transactions,
           validity: Some(Validity
            {
                refund: None,
                refund_config: Some({ vec![RefundConfig{
                    
                    address: Address::from_str("0x40").unwrap(),
                    percent:30,

                   }]
            
                }),
            
            }),
            
            privacy: Some(Privacy
            {
                hints: Some(PrivacyHint
                {
                    calldata: false,
                    contract_address: false,
                    logs: false,
                    function_selector: false,
                    hash: false,
                    tx_hash: false,
                }), 

                builders: Some(vec![
                    Address::from_str("0x1f9090aaE28b8a3dCeaDf281B0F12828e676c326").unwrap(), //rysnc builder
                    Address::from_str("0x690B9A9E9aa1C9dB991C7721a92d351Db4FaC990").unwrap(), //builder0x69
                    Address::from_str("0x95222290DD7278Aa3Ddd389Cc1E1d165CC4BAfe5").unwrap(), //beaverbuild
                    Address::from_str("0xDAFEA492D9c6733ae3d56b7Ed1ADB60692c98Bc5").unwrap(), //Flashbot builder
                    Address::from_str("0x4838B106FCe9647Bdf1E7877BF73cE8B0BAD5f97").unwrap(), //Titan builder
                    

                ]),

            }),
        }
    }

    /// Helper function to create a simple bundle request with sensible defaults (bundle is valid for the next 5 blocks).
    pub fn make_simple(block_num: U64, transactions: Vec<BundleTx>) -> Self {
        // bundle is valid for 5 blocks
        let max_block = block_num.saturating_add(U64::from(30));
        Self::new(
            block_num,
            Some(max_block),
            ProtocolVersion::Beta1,
            transactions,
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::types::BundleRequest;

    #[test]
    fn can_deserialize() {
        let str = r#"
        [{
            "version": "v0.1",
            "inclusion": {
                "block": "0x1"
            },
            "body": [{
                "tx": "0x02f86b0180843b9aca00852ecc889a0082520894c87037874aed04e51c29f582394217a0a2b89d808080c080a0a463985c616dd8ee17d7ef9112af4e6e06a27b071525b42182fe7b0b5c8b4925a00af5ca177ffef2ff28449292505d41be578bebb77110dfc09361d2fb56998260",
                "canRevert": false
            }]
        }]
        "#;
        let res: Result<Vec<BundleRequest>, _> = serde_json::from_str(str);
        assert!(res.is_ok());
    }
}
