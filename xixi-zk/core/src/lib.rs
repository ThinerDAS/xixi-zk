use serde::{Serialize, Deserialize};
use rkyv::{Archive, Serialize as RkyvSerialize, Deserialize as RkyvDeserialize};

/// Game configuration data structure (matches convert_motadata.py)
#[derive(Debug, Serialize, Deserialize, Archive, RkyvSerialize, RkyvDeserialize)]
#[archive(check_bytes)]
#[archive_attr(derive(Debug))]
pub struct GameConfig {
    pub major_adj: Vec<Vec<u32>>,
    pub major_minor_adj: Vec<Vec<u32>>,
    pub major_desc: Vec<MajorDesc>,
    pub minor_desc: Vec<MinorDesc>,
    pub enemy_data: Vec<Enemy>,
    pub init_stat: PlayerState,
    pub levelup_desc: Vec<LevelUp>,
}

/// Major node effects
#[derive(Debug, Serialize, Deserialize, Archive, RkyvSerialize, RkyvDeserialize)]
#[archive(check_bytes)]
#[archive_attr(derive(Debug))]
pub enum MajorDesc {
    Enemy(u32),
    Delta(Vec<(AttrType, i32)>),
}

/// Attribute type enum
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Archive, RkyvSerialize, RkyvDeserialize)]
#[archive(check_bytes)]
#[archive_attr(derive(Debug))]
pub enum AttrType {
    Hp,
    Atk,
    Def,
    Mdef,
    Exp,
    Lv,
    Salt,
    BigSalt
}

impl AttrType {
    pub fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            AttrType::Hp => serializer.serialize_str("hp"),
            AttrType::Atk => serializer.serialize_str("atk"),
            AttrType::Def => serializer.serialize_str("def"),
            AttrType::Mdef => serializer.serialize_str("mdef"),
            AttrType::Exp => serializer.serialize_str("exp"),
            AttrType::Lv => serializer.serialize_str("lv"),
            AttrType::Salt => serializer.serialize_str("salt"),
            AttrType::BigSalt => serializer.serialize_str("big_salt"),
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = <String as serde::Deserialize>::deserialize(deserializer)?;
        match s.as_str() {
            "hp" => Ok(AttrType::Hp),
            "atk" => Ok(AttrType::Atk),
            "def" => Ok(AttrType::Def),
            "mdef" => Ok(AttrType::Mdef),
            "exp" => Ok(AttrType::Exp),
            "lv" => Ok(AttrType::Lv),
            "salt" => Ok(AttrType::Salt),
            "big_salt" => Ok(AttrType::BigSalt),
            _ => Err(serde::de::Error::custom("invalid attribute type")),
        }
    }
}

/// Minor node reward description
#[derive(Debug, Serialize, Deserialize, Clone, Archive, RkyvSerialize, RkyvDeserialize)]
#[archive(check_bytes)]
#[archive_attr(derive(Debug))]
pub struct MinorDesc {
    pub atk: i32,
    pub def: i32,
    pub hp: i32,
    pub mdef: i32,
}

/// Enemy definition 
#[derive(Debug, Serialize, Deserialize, Clone, Archive, RkyvSerialize, RkyvDeserialize)]
#[archive(check_bytes)]
#[archive_attr(derive(Debug))]
pub struct Enemy {
    pub atk: i32,
    pub def: i32,
    pub hp: i32,
    pub attimes: i32,
    pub exp: i32,
    pub magic: bool,
    pub solid: bool,
    pub speedy: bool,
    pub nobomb: bool,
}

/// Player initial state
#[derive(Debug, Serialize, Deserialize, Clone, Archive, RkyvSerialize, RkyvDeserialize)]
#[archive(check_bytes)]
#[archive_attr(derive(Debug))]
pub struct PlayerState {
    pub hp: i32,
    pub atk: i32,
    pub def: i32,
    pub mdef: i32,
    pub exp: i32,
    pub lv: u32,
    pub salt: i32,
    pub big_salt: i32,
}

/// Level up requirements
#[derive(Debug, Serialize, Deserialize, Archive, RkyvSerialize, RkyvDeserialize)]
#[archive(check_bytes)]
#[archive_attr(derive(Debug))]
pub struct LevelUp {
    pub minor: u32,
    pub need: i32,
    pub clear: bool,
}

impl GameConfig {
    /// Load config from JSON (compatible with convert_motadata.py output)
    pub fn from_json(json_str: &str) -> Result<Self, serde_json::Error> {
        let config: Self = serde_json::from_str(json_str)?;
        Ok(config)
    }
}

impl GameConfig {
    /// Serialize to rkyv bytes with proper error handling
    pub fn to_rkyv(&self) -> Vec<u8> {
        rkyv::to_bytes::<_, 256>(self)
            .expect("Failed to serialize GameConfig")
            .to_vec()
    }

    /// Deserialize from rkyv bytes with validation
    pub fn from_rkyv(bytes: &[u8]) -> Self {
        rkyv::from_bytes(bytes)
            .expect("Failed to deserialize GameConfig")
    }
}

/// Output structure containing all verification data
#[derive(Debug, Serialize, Deserialize)]
pub struct Output {
    pub config_hash: [u8; 32],
    pub user_cred_hash: [u8; 32],
    pub scores: Vec<i64>,
}