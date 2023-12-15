use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum VpnPeerConfig {
    Disabled,
    #[serde(rename_all = "kebab-case")]
    Netbird {
        management_url: Url,
        setup_key: netbird::SetupKey,
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum HttpsOnly {
    True,
    False,
}
impl HttpsOnly {
    pub fn to_bool(&self) -> bool {
        match self {
            HttpsOnly::True => true,
            HttpsOnly::False => false,
        }
    }
    pub fn default() -> Self {
        HttpsOnly::True
    }
}

pub mod netbird {
    use serde::{Deserialize, Serialize};
    use uuid::Uuid;

    #[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
    #[serde(transparent)]
    pub struct SetupKey { pub uuid: Uuid }
    impl SetupKey {
        pub fn random() -> Self {
            Self { uuid: Uuid::new_v4() }
        }
    }
    impl From<Uuid> for SetupKey {
        fn from(value: Uuid) -> Self {
            Self { uuid: value }
        }
    }
}
