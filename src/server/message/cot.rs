//!
//! # Cause and diraction of transmission.
//! Bitmask (Enum) being a part of the [Point](../point/index.html).  
//! Contains information about transmission cause and direction.  
//! Basic values at the moment (can be extended):
//! ```ignore
//! Inf      = 0b_0000_0010; // 2   (0x2);
//! Act      = 0b_0000_0100; // 4   (0x4);
//! ActCon   = 0b_0000_1000; // 8   (0x8);
//! ActErr   = 0b_0001_0000; // 16  (0x10);
//! Req      = 0b_0010_0000; // 32  (0x20);
//! ReqCon   = 0b_0100_0000; // 64  (0x40);
//! ReqErr   = 0b_1000_0000; // 128 (0x80);
//! ```
use sal_core::error::Error;
use serde::{Serialize, Deserialize};
///
/// Cause and diraction of the transmission
/// - `Inf` - Information (Informational message, in general sent by backend to the client)
/// - `Act` - Activation (Command message, response is not required, optionally my be sent Cot::ActCon / Cot::ActErr)
/// - `ActCon` - Activation confirmatiom
/// - `ActErr` - Activation error
/// - `Req` - Request (Request message, Client expects response with Cot::ReqCon / Cot::ReqErr)
/// - `ReqCon` - Rquest | Confirmatiom reply 
/// - `ReqErr` - Rquest | Error reply
#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq, PartialOrd, Serialize, Deserialize, bincode::Encode, bincode::Decode)]
#[repr(u8)]
pub enum Cot {
    #[serde(rename = "Inf")]
    #[serde(alias = "inf", alias = "Inf", alias = "INF")]
    Inf = 0b00000010,
    #[serde(rename = "Act")]
    #[serde(alias = "act", alias = "Act", alias = "ACT")]
    Act = 0b00000100,
    #[serde(rename = "ActCon")]
    #[serde(alias = "actcon", alias = "ActCon", alias = "ACTCON")]
    ActCon = 0b00001000,
    #[serde(rename = "ActErr")]
    #[serde(alias = "acterr", alias = "ActErr", alias = "ACTERR")]
    ActErr = 0b00010000,
    #[serde(rename = "Req")]
    #[serde(alias = "req", alias = "Req", alias = "REQ")]
    Req = 0b00100000,
    #[serde(rename = "ReqCon")]
    #[serde(alias = "reqcon", alias = "ReqCon", alias = "REQCON")]
    ReqCon = 0b01000000,
    #[serde(rename = "ReqErr")]
    #[serde(alias = "reqerr", alias = "ReqErr", alias = "REQERR")]
    ReqErr = 0b10000000,
}
//
// 
impl Cot {
    ///
    /// Returns true if [self] contains `rhs`
    pub fn contains(&self, rhs: Cot) -> bool {
        (*self & rhs) > 0
    }
    ///
    /// Returns string representation of the given [Cot]
    pub fn as_str(&self) -> &str {
        match self {
            Cot::Inf => "Inf",
            Cot::Act => "Act",
            Cot::ActCon => "ActCon",
            Cot::ActErr => "ActErr",
            Cot::Req => "Req",
            Cot::ReqCon => "ReqCon",
            Cot::ReqErr => "ReqErr",
        }
    }
    ///
    /// Returns `u8` representation of the given [Cot]
    pub fn as_u8(&self) -> u8 {
        *self as u8
    }
    ///
    /// Returns &[u8] representation of the given [Cot]
    pub fn as_bytes(&self) -> [u8; 1] {
        [*self as u8]
    }
    pub fn from_be_bytes(bytes: &[u8]) -> Result<Self, Error> {
        match bytes {
            [0b00000010] => Ok(Self::Inf),
            [0b00000100] => Ok(Self::Act),
            [0b00001000] => Ok(Self::ActCon),
            [0b00010000] => Ok(Self::ActErr),
            [0b00100000] => Ok(Self::Req),
            [0b01000000] => Ok(Self::ReqCon),
            [0b10000000] => Ok(Self::ReqErr),
            _ => Err(Error::new("Cot", "from_be_bytes").err(format!("Can't parse 'Cot' from bytes: {:?}", &bytes[..16])))
        }
    }
    ///
    /// Returns `Ok` reply [Cot] to current
    pub fn reply_ok(&self) -> Self {
        match self {
            Cot::Act => Cot::ActCon,
            Cot::Req => Cot::ReqCon,
            _ => {
                log::warn!("{}", Error::new("Cot", "reply_ok").err(format!("Can't find ok reply 'Cot' for {:?}", self)));
                *self
            }
        }
    }
    ///
    /// Returns `Err` reply [Cot] to current
    pub fn reply_err(&self) -> Self {
        match self {
            Cot::Act => Cot::ActErr,
            Cot::Req => Cot::ReqErr,
            _ => {
                log::warn!("{}", Error::new("Cot", "reply_err").err(format!("Can't find error reply 'Cot' for {:?}", self)));
                *self
            }
        }
    }
}

//
// 
impl Default for Cot {
    fn default() -> Self {
        Self::Inf
    }
}
//
// 
impl AsRef<str> for Cot {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
//
// 
impl std::ops::BitOr for Cot {
    type Output = u32;
    fn bitor(self, rhs: Cot) -> Self::Output {
        self as u32 | rhs as u32
    }
}
//
// 
impl std::ops::BitAnd for Cot {
    type Output = u32;
    fn bitand(self, rhs: Cot) -> Self::Output {
        self as u32 & rhs as u32
    }
}
//
// 
impl std::ops::BitOr<Cot> for u32 {
    type Output = u32;
    fn bitor(self, rhs: Cot) -> Self::Output {
        self | rhs as u32
    }
}
//
// 
impl std::ops::BitAnd<Cot> for u32 {
    type Output = u32;
    fn bitand(self, rhs: Cot) -> Self::Output {
        self & rhs as u32
    }
}
//
// 
impl std::fmt::Binary for Cot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&format!("{:#08b}",self.to_owned() as u32), f)
    }
}
