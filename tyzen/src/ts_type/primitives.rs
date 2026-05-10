use std::{
    net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddrV4, SocketAddrV6},
    num::{
        NonZeroI8, NonZeroI16, NonZeroI32, NonZeroI64, NonZeroI128, NonZeroIsize, NonZeroU8,
        NonZeroU16, NonZeroU32, NonZeroU64, NonZeroU128, NonZeroUsize,
    },
    os::unix::net::SocketAddr,
    path::{Path, PathBuf},
    time::{Duration, SystemTime},
};

use crate::TsType;

macro_rules! impl_ts_type {
    ($($($t:ty),+ => $ts:literal),* $(,)?) => {
        $($(
            impl TsType for $t {
                fn ts_name() -> String {
                    $ts.to_string()
                }
            }
        )*)*
    };
}

impl_ts_type! {
    u8, i8, NonZeroU8, NonZeroI8,
    u16, i16, NonZeroU16, NonZeroI16,
    u32, i32, NonZeroU32, NonZeroI32,
    u64, i64, NonZeroU64, NonZeroI64,
    u128, i128, NonZeroU128, NonZeroI128,
    usize, isize, NonZeroUsize, NonZeroIsize,
    f32, f64 => "number",
    bool => "boolean",
    char, Path, PathBuf, String, str,
    Ipv4Addr, Ipv6Addr, IpAddr, SocketAddrV4, SocketAddrV6, SocketAddr,
    Duration, SystemTime => "string",
    () => "null",
}
