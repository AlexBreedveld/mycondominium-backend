use super::prelude::*;
use user_agent_parser::UserAgentParser;

#[derive(
    Queryable,
    Selectable,
    Insertable,
    Serialize,
    Deserialize,
    Clone,
    Debug,
    AsChangeset,
    Validate,
    ToSchema,
    DbOps,
)]
#[diesel(table_name = crate::schema::auth_tokens)]
pub struct AuthTokenModel {
    pub user_id: Uuid,
    pub id: Uuid,
    pub time_added: NaiveDateTime,
    pub active: bool,
    pub time_last_used: NaiveDateTime,
    pub device: Option<String>,
    pub browser: Option<String>,
    pub version: Option<String>,
    pub cpu_arch: Option<String>,
}

pub trait FromUaParser: Sized {
    type ParserType<'a>;
    fn from_ua_parser(s: Self::ParserType<'_>) -> Self;
    fn parse(s: &'static str, parser: &'static UserAgentParser) -> Self;
}

pub trait UaParser {
    fn parse_cpu_serde(&self, s: String) -> UaCPU
    where
        Self: Sized;
    fn parse_os_serde(s: String) -> UaOS;
    fn parse_device_serde(s: String) -> UaDevice;
    fn parse_engine_serde(s: String) -> UaEngine;
    fn parse_product_serde(s: String) -> UaProduct;
}

#[derive(Serialize, Deserialize, Clone)]
pub struct UserAgent {
    pub cpu: UaCPU,
    pub os: UaOS,
    pub device: UaDevice,
    pub engine: UaEngine,
    pub product: UaProduct,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct UaCPU {
    pub architecture: Option<String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct UaOS {
    pub name: Option<String>,
    pub major: Option<String>,
    pub minor: Option<String>,
    pub patch: Option<String>,
    pub patch_minor: Option<String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct UaDevice {
    pub name: Option<String>,
    pub brand: Option<String>,
    pub model: Option<String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct UaEngine {
    pub name: Option<String>,
    pub major: Option<String>,
    pub minor: Option<String>,
    pub patch: Option<String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct UaProduct {
    pub name: Option<String>,
    pub major: Option<String>,
    pub minor: Option<String>,
    pub patch: Option<String>,
}

impl FromUaParser for UaCPU {
    type ParserType<'a> = user_agent_parser::CPU<'a>;

    fn from_ua_parser(s: Self::ParserType<'_>) -> Self {
        match s.architecture {
            Some(a) => UaCPU {
                architecture: Some(a.to_string()),
            },
            None => UaCPU { architecture: None },
        }
    }

    fn parse(s: &'static str, parser: &'static UserAgentParser) -> Self {
        let cpu = parser.parse_cpu(s);
        UaCPU::from_ua_parser(cpu)
    }
}

impl FromUaParser for UaOS {
    type ParserType<'a> = user_agent_parser::OS<'a>;

    fn from_ua_parser(s: Self::ParserType<'_>) -> Self {
        UaOS {
            name: s.name.map(|n| n.to_string()),
            major: s.major.map(|n| n.to_string()),
            minor: s.minor.map(|n| n.to_string()),
            patch: s.patch.map(|n| n.to_string()),
            patch_minor: s.patch_minor.map(|n| n.to_string()),
        }
    }

    fn parse(s: &'static str, parser: &'static UserAgentParser) -> Self {
        let os = parser.parse_os(s);
        UaOS::from_ua_parser(os)
    }
}

impl FromUaParser for UaDevice {
    type ParserType<'a> = user_agent_parser::Device<'a>;

    fn from_ua_parser(s: Self::ParserType<'_>) -> Self {
        UaDevice {
            name: s.name.map(|n| n.to_string()),
            brand: s.brand.map(|n| n.to_string()),
            model: s.model.map(|n| n.to_string()),
        }
    }

    fn parse(s: &'static str, parser: &'static UserAgentParser) -> Self {
        let device = parser.parse_device(s);
        UaDevice::from_ua_parser(device)
    }
}

impl FromUaParser for UaEngine {
    type ParserType<'a> = user_agent_parser::Engine<'a>;

    fn from_ua_parser(s: Self::ParserType<'_>) -> Self {
        UaEngine {
            name: s.name.map(|n| n.to_string()),
            major: s.major.map(|n| n.to_string()),
            minor: s.minor.map(|n| n.to_string()),
            patch: s.patch.map(|n| n.to_string()),
        }
    }

    fn parse(s: &'static str, parser: &'static UserAgentParser) -> Self {
        let engine = parser.parse_engine(s);
        UaEngine::from_ua_parser(engine)
    }
}

impl FromUaParser for UaProduct {
    type ParserType<'a> = user_agent_parser::Product<'a>;

    fn from_ua_parser(s: Self::ParserType<'_>) -> Self {
        UaProduct {
            name: s.name.map(|n| n.to_string()),
            major: s.major.map(|n| n.to_string()),
            minor: s.minor.map(|n| n.to_string()),
            patch: s.patch.map(|n| n.to_string()),
        }
    }

    fn parse(s: &'static str, parser: &'static UserAgentParser) -> Self {
        let product = parser.parse_product(s);
        UaProduct::from_ua_parser(product)
    }
}
