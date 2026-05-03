use enumflags2::bitflags;
use serde::{Deserialize, Serialize};
use zbus::zvariant::Type;

#[bitflags]
#[repr(u32)]
#[derive(Type, Debug, PartialEq, Eq, Copy, Clone, Serialize, Deserialize)]
pub enum RegisterFlags {
    NONE,
    RunInThread,
}
