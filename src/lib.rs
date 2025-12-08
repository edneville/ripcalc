use cdb::CDB;
use dns_lookup::{lookup_addr, lookup_host};
use maxminddb::*;
use nix::ifaddrs::*;
use nix::sys::socket::AddressFamily;
use nix::sys::socket::SockaddrLike;
use nix::sys::socket::SockaddrStorage;
use nix::sys::stat::fstat;
use nix::sys::stat::SFlag;
use regex::*;
use reqwest::header::HeaderMap;
use serde_json::*;
use std::cell::RefCell;
use std::collections::HashMap;
use std::convert::TryInto;
use std::fmt;
use std::io::BufRead;
use std::net::Ipv4Addr;
use std::net::Ipv6Addr;
use std::os::fd::BorrowedFd;
use std::str::FromStr;
use std::sync::Arc;
use std::net::IpAddr;

#[derive(Debug, PartialEq, PartialOrd, Hash, Eq, Clone)]
pub enum Addr {
    V6(Ipv6Addr),
    V4(Ipv4Addr),
}

#[derive(Debug, PartialEq, PartialOrd, Hash, Eq, Clone)]
pub struct Ip {
    pub address: Addr,
    pub cidr: u32,
}

pub struct NetRow {
    pub row: HashMap<String, String>,
}

#[derive(Clone)]
pub struct Config {
    pub interface_names: Vec<InterfaceAddress>,
    pub hm: HashMap<String, String>,
    pub used: Option<HashMap<Ip, bool>>,
    pub input_family: Option<InputFamily>,
    pub net_used: HashMap<Ip, HashMap<Ip, bool>>,
    pub options: HashMap<String, String>,
    pub cdb: Option<Arc<CDB>>,
    pub count: Option<HashMap<Ip, usize>>,
    pub abuse_cache: Option<HashMap<Ip, String>>,
    pub mmasn: Option<Arc<maxminddb::Reader<Vec<u8>>>>,
    pub mmcity: Option<Arc<maxminddb::Reader<Vec<u8>>>>,
    pub mmcountry: Option<Arc<maxminddb::Reader<Vec<u8>>>>,
}

#[derive(Debug, Clone)]
pub enum InputFamily {
    Any,
    IPv4,
    IPv6,
}

pub enum FormatMode {
    Text,
    Binary,
    SplitBinary,
    Integer,
    SignedInteger,
    Hex,
    Backslash,
}

pub enum FormatProcessor {
    Percent,
    Backslash,
    None,
}

pub struct FormatDetail<'a> {
    pub ip: Option<&'a Ip>,
    pub line: Option<&'a String>,
}

impl<'a> FormatDetail<'a> {
    pub fn new(ip: Option<&'a Ip>, line: Option<&'a String>) -> FormatDetail<'a> {
        FormatDetail { ip, line }
    }
}

pub enum Reverse {
    None,
    Input,
    Source,
    Both,
}

pub enum MaskType {
    Network,
    Broadcast,
    Wildcard,
    SubnetMask,
}

impl Config {
    pub fn new() -> Config {
        Config {
            interface_names: vec![],
            hm: HashMap::new(),
            used: None,
            input_family: None,
            net_used: HashMap::new(),
            options: HashMap::new(),
            cdb: None,
            count: None,
            abuse_cache: None,
            mmasn: None,
            mmcity: None,
            mmcountry: None,
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for Config {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> std::result::Result<(), fmt::Error> {
        #[derive(Debug)]
        #[allow(dead_code)]
        struct Config<'a> {
            interface_names: &'a Vec<InterfaceAddress>,
            hm: &'a HashMap<String, String>,
            used: &'a Option<HashMap<Ip, bool>>,
            input_family: &'a Option<InputFamily>,
            net_used: &'a HashMap<Ip, HashMap<Ip, bool>>,
            options: &'a HashMap<String, String>,
            count: &'a Option<HashMap<Ip, usize>>,
            abuse_cache: &'a Option<HashMap<Ip, String>>,
            mmasn: &'a Option<Arc<maxminddb::Reader<Vec<u8>>>>,
            mmcity: &'a Option<Arc<maxminddb::Reader<Vec<u8>>>>,
            mmcountry: &'a Option<Arc<maxminddb::Reader<Vec<u8>>>>,
        }

        let Self {
            interface_names,
            hm,
            used,
            input_family,
            net_used,
            options,
            cdb: _,
            count,
            abuse_cache,
            mmasn,
            mmcity,
            mmcountry,
        } = self;

        fmt::Debug::fmt(
            &Config {
                interface_names,
                hm,
                used,
                input_family,
                net_used,
                options,
                count,
                abuse_cache,
                mmasn,
                mmcity,
                mmcountry,
            },
            f,
        )
    }
}

impl Ip {
    pub fn num_representation(&self) -> String {
        match self.address {
            Addr::V4(x) => u32::from(x).to_string(),
            Addr::V6(x) => u128::from(x).to_string(),
        }
    }
    pub fn signed_num_representation(&self) -> String {
        match self.address {
            Addr::V4(x) => {
                let n = u32::from(x) as i32;
                n.to_string()
            }
            Addr::V6(x) => {
                let n = u128::from(x) as i128;
                n.to_string()
            }
        }
    }
    pub fn bin_representation(&self) -> String {
        match self.address {
            Addr::V4(x) => format!("{:032b}", u32::from(x)),
            Addr::V6(x) => format!("{:0128b}", u128::from(x)),
        }
    }
    pub fn bin_split_representation(&self) -> String {
        let mut s = match self.address {
            Addr::V4(x) => format!("{:032b}", u32::from(x)),
            Addr::V6(x) => format!("{:0128b}", u128::from(x)),
        };
        s.insert(self.cidr as usize, ' ');
        s
    }
    pub fn hex_quad_representation(&self) -> String {
        match self.address {
            Addr::V4(x) => {
                let mut s: String = "".to_string();
                let octet = x.octets();
                for o in &octet {
                    s.push_str(&format!("{:02x}", o));
                }
                s.to_string()
            }
            Addr::V6(x) => {
                let mut s: String = "".to_string();
                let octet = x.octets();
                for (i, o) in octet.iter().enumerate() {
                    s.push_str(&format!("{:02x}", o));
                    if i % 2 == 1 && i != 15 {
                        s.push(':');
                    }
                }
                s.to_string()
            }
        }
    }
}

pub fn host_address(ip: Addr) -> Ip {
    match ip {
        Addr::V4(_) => Ip {
            address: ip,
            cidr: 32,
        },
        Addr::V6(_) => Ip {
            address: ip,
            cidr: 128,
        },
    }
}

pub fn parse_mask(mask: &str) -> Option<u32> {
    mask.parse::<u32>().ok()
}

pub fn parse_v6(address: &str, input_base: Option<i32>, reverse: bool) -> Option<Addr> {
    match input_base {
        Some(base) => Some(Addr::V6(Ipv6Addr::from(if base < 0 {
            let base = -base;
            match i128::from_str_radix(address, base as u32) {
                Ok(y) => y as u128,
                Err(_e) => {
                    return None;
                }
            }
        } else {
            match u128::from_str_radix(address, base as u32) {
                Ok(y) => y,
                Err(_e) => {
                    return None;
                }
            }
        }))),
        None => match Ipv6Addr::from_str(address) {
            Ok(mut i) => {
                if reverse {
                    let mut j = i.octets();
                    j.reverse();
                    for i in &mut j {
                        *i = ((*i & 0x0f) << 4) | (*i & 0xf0) >> 4;
                    }

                    i = Ipv6Addr::from(j);
                }
                Some(Addr::V6(i))
            }
            Err(_) => None,
        },
    }
}

pub fn parse_v4(address: &str, input_base: Option<i32>, reverse: bool) -> Option<Addr> {
    match input_base {
        Some(base) => {
            let mut address = address;
            let mut arr: Vec<&str>;
            let a;
            if reverse && address.find('.').is_some() {
                arr = address.split('.').collect();
                arr.reverse();
                a = arr.join(".");
                address = &a;
            }

            if address.find('.').is_none()
                && !reverse
                && input_base.is_some()
                && input_base.unwrap() != 16
            {
                return Some(Addr::V4(Ipv4Addr::from(if base < 0 {
                    let base = -base;
                    match i32::from_str_radix(address, base as u32) {
                        Ok(y) => y as u32,
                        Err(_e) => {
                            return None;
                        }
                    }
                } else {
                    match u32::from_str_radix(address, base as u32) {
                        Ok(y) => y,
                        Err(_e) => {
                            return None;
                        }
                    }
                })));
            }

            let parts: Vec<String>;
            let chars: Vec<char>;

            let split = if address.find('.').is_none() {
                chars = address.chars().collect();

                chars
                    .chunks(2)
                    .map(|c| c.iter().collect::<String>())
                    .collect::<Vec<_>>()
            } else {
                parts = address.split('.').map(|s| s.to_string()).collect();
                parts
            };

            let mut arr: [u8; 4] = [0, 0, 0, 0];

            for (x, y) in split.iter().enumerate() {
                let base = if base < 0 { -base } else { base };
                arr[x] = match u8::from_str_radix(y, base as u32) {
                    Ok(y) => y,
                    Err(e) => {
                        println!("cannot convert {}: {}", y, e);
                        return None;
                    }
                }
            }

            if reverse {
                arr.reverse();
            }

            Some(Addr::V4(Ipv4Addr::from(arr)))
        }
        None => match Ipv4Addr::from_str(address) {
            Ok(mut i) => {
                if reverse {
                    let mut j = i.octets();
                    j.reverse();
                    i = Ipv4Addr::from(j);
                }
                Some(Addr::V4(i))
            }
            Err(_) => None,
        },
    }
}

pub fn parse_v4_v6(address: &str, input_base: Option<i32>, reverse: bool) -> Option<Addr> {
    if address.find(':').is_some() || (address.len() > 20 && input_base.is_some()) {
        return parse_v6(address, input_base, reverse);
    }

    parse_v4(address, input_base, reverse)
}

pub fn parse_address_mask(
    a: &str,
    default_v4_mask: Option<u32>,
    default_v6_mask: Option<u32>,
    input_base: Option<i32>,
    reverse: bool,
    config: &RefCell<Config>,
) -> Option<Ip> {
    let parts: Vec<&str> = a.split('/').collect();

    let mut arg = parts[0];

    let mut input_mask: Option<u32> = None;
    if parts.len() > 1 {
        if let Some(m) = parse_mask(parts[1]) {
            input_mask = Some(m);
        }
    };

    let input_ip = match config.borrow().input_family {
        Some(InputFamily::Any) | None => parse_v4_v6(arg, input_base, reverse),
        Some(InputFamily::IPv6) => parse_v6(arg, input_base, reverse),
        Some(InputFamily::IPv4) => parse_v4(arg, input_base, reverse),
    };

    if let Some(input_ip) = input_ip {
        return Some(Ip {
            address: input_ip.clone(),
            cidr: match input_ip {
                Addr::V4(_) => input_mask.unwrap_or_else(|| default_v4_mask.unwrap_or(32)),
                Addr::V6(_) => input_mask.unwrap_or_else(|| default_v6_mask.unwrap_or(128)),
            },
        });
    }

    for p in ["https://", "http://", "ftp://", "sftp://", "ftps://"] {
        if a.starts_with(p) {
            let v: Vec<&str> = a.split('/').collect();
            arg = v[2];
            break;
        }
    }

    if !is_like_hostname(arg) {
        return None;
    }

    let arg = ip_lookup(arg, &mut config.borrow_mut().hm);

    let input_ip = parse_v4_v6(&arg, input_base, reverse);

    if let Some(input_ip) = input_ip {
        return Some(Ip {
            address: input_ip.clone(),
            cidr: match input_ip {
                Addr::V4(_) => input_mask.unwrap_or(24),
                Addr::V6(_) => input_mask.unwrap_or(64),
            },
        });
    }

    None
}

pub fn network_iter(ip: &Ip) -> impl std::iter::Iterator<Item = Ip> + '_ {
    mask_iter(ip, &MaskType::Network)
}

pub fn broadcast_iter(ip: &Ip) -> impl std::iter::Iterator<Item = Ip> + '_ {
    mask_iter(ip, &MaskType::Broadcast)
}

pub fn wildcard_iter(ip: &Ip) -> impl std::iter::Iterator<Item = Ip> + '_ {
    mask_iter(ip, &MaskType::Wildcard)
}

pub fn subnet_iter(ip: &Ip) -> impl std::iter::Iterator<Item = Ip> + '_ {
    mask_iter(ip, &MaskType::SubnetMask)
}

pub fn mask_iter<'a>(
    ip: &'a Ip,
    mask_type: &'a MaskType,
) -> impl std::iter::Iterator<Item = Ip> + 'a {
    let mut cidr: i32 = ip.cidr.try_into().unwrap();

    std::iter::from_fn(move || {
        if cidr == -1 {
            None
        } else {
            cidr -= 1;
            Some(match mask_type {
                MaskType::Network => network(&Ip {
                    address: ip.address.clone(),
                    cidr: (cidr + 1) as u32,
                }),
                MaskType::Broadcast => broadcast(&Ip {
                    address: ip.address.clone(),
                    cidr: (cidr + 1) as u32,
                }),
                MaskType::SubnetMask => subnet(&Ip {
                    address: ip.address.clone(),
                    cidr: (cidr + 1) as u32,
                }),
                MaskType::Wildcard => wildcard(&Ip {
                    address: ip.address.clone(),
                    cidr: (cidr + 1) as u32,
                }),
            })
        }
    })
}

pub fn addresses<'a>(
    ip: &'a Ip,
    used: Option<&'a HashMap<Addr, bool>>,
    mask: Option<u32>,
) -> impl std::iter::Iterator<Item = Ip> + 'a {
    let b = broadcast(ip);
    let mut net = network(ip);

    let mut last = false;

    std::iter::from_fn(move || {
        if let Addr::V4(x) = net.address {
            if let Addr::V4(y) = b.address {
                let adder = match mask {
                    Some(x) => {
                        net.cidr = x;
                        let base: u32 = 2;
                        base.pow(32 - x)
                    }
                    None => 1,
                };

                let mut p = u32::from(x);
                let q = u32::from(y);

                loop {
                    net = Ip {
                        address: Addr::V4(Ipv4Addr::from(p + adder)),
                        cidr: net.cidr,
                    };

                    let i = Ip {
                        address: Addr::V4(Ipv4Addr::from(p)),
                        cidr: net.cidr,
                    };

                    if last {
                        return None;
                    }

                    let diff = q - p;
                    if diff < adder {
                        last = true;
                    }

                    if let Some(map) = &used {
                        if map.get(&Addr::V4(Ipv4Addr::from(p))).is_some() {
                            p += adder;

                            continue;
                        }
                    }

                    return Some(if mask.is_none() { i } else { network(&i) });
                }
            }
        }

        if let Addr::V6(x) = net.address {
            if let Addr::V6(y) = b.address {
                let adder = match mask {
                    Some(x) => {
                        net.cidr = x;
                        let base: u128 = 2;
                        base.pow(128 - x)
                    }
                    None => 1,
                };

                let mut p = u128::from(x);
                let q = u128::from(y);

                loop {
                    net = Ip {
                        address: Addr::V6(Ipv6Addr::from(p + adder)),
                        cidr: net.cidr,
                    };

                    let i = Ip {
                        address: Addr::V6(Ipv6Addr::from(p)),
                        cidr: net.cidr,
                    };

                    if last {
                        return None;
                    }

                    let diff = q - p;
                    if diff < adder {
                        last = true;
                    }

                    if let Some(map) = &used {
                        if map.get(&Addr::V6(Ipv6Addr::from(p))).is_some() {
                            p += adder;

                            continue;
                        }
                    }

                    return Some(if mask.is_none() { i } else { network(&i) });
                }
            }
        }

        Some(net.clone())
    })
}

pub fn smallest_group_network(networks: &HashMap<Ip, bool>) -> Option<Ip> {
    if networks.is_empty() {
        return None;
    }

    let ip = networks.keys().next().unwrap();
    let mut ip = Ip {
        address: match ip.address {
            Addr::V4(x) => Addr::V4(x),
            Addr::V6(x) => Addr::V6(x),
        },
        cidr: ip.cidr,
    };

    for key in networks.keys().skip(1) {
        let mut key = key.clone();
        match (&ip.address, &key.address) {
            (Addr::V4(_), Addr::V4(_)) | (Addr::V6(_), Addr::V6(_)) => {
                if key.cidr < ip.cidr {
                    ip.cidr = key.cidr;
                }
                key.cidr = ip.cidr;
                while network(&key) != network(&ip) {
                    if ip.cidr == 0 {
                        return None;
                    }
                    ip.cidr -= 1;
                    key.cidr = ip.cidr;
                }
                ip = network(&ip);
            }
            (_, _) => {
                return None;
            }
        }
    }

    Some(ip)
}

pub fn smallest_group_network_limited(
    networks: &HashMap<Ip, bool>,
    cidr4: u32,
    cidr6: u32,
) -> Option<Vec<Ip>> {
    if networks.is_empty() {
        return None;
    }

    let mut net_list: HashMap<Ip, Ip> = HashMap::new();

    for key in networks.keys() {
        let cidr;
        let mut bucket_ip = key.clone();
        bucket_ip.cidr = if let Addr::V4(_) = key.address {
            cidr = cidr4;
            cidr4
        } else {
            cidr = cidr6;
            cidr6
        };

        let mut key_copy = key.clone();

        // don't add a entry with a bigger network than cidr
        if key_copy.cidr < cidr {
            key_copy.cidr = cidr;
        }

        let bucket = network(&bucket_ip);

        if !net_list.contains_key(&bucket) {
            net_list.insert(bucket.clone(), key_copy.clone());
        }

        let ip = net_list.get_mut(&bucket).unwrap();

        match (&ip.address, &key.address) {
            (Addr::V4(_), Addr::V4(_)) | (Addr::V6(_), Addr::V6(_)) => {
                // println!("ip.cidr {}, key_copy.cidr {}", ip.cidr, key_copy.cidr);
                if key_copy.cidr < ip.cidr {
                    // println!("key less than ip: ip.cidr {}, key_copy.cidr {}", ip.cidr, key_copy.cidr);
                    ip.cidr = key_copy.cidr;
                }
                key_copy.cidr = ip.cidr;
                while network(&key_copy) != network(ip) && ip.cidr > cidr {
                    if ip.cidr == 0 {
                        return None;
                    }
                    ip.cidr -= 1;
                    key_copy.cidr = ip.cidr;
                }
                *ip = network(ip);
            }
            (_, _) => {
                return None;
            }
        }
    }

    Some(net_list.values().cloned().collect())
}

impl fmt::Display for Ip {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.address {
            Addr::V4(x) => {
                write!(f, "{}", x)
            }
            Addr::V6(x) => {
                write!(f, "{}", x)
            }
        }
    }
}

pub fn broadcast(ip: &Ip) -> Ip {
    match ip.address {
        Addr::V4(x) => {
            let mut bin: u32 = 0;
            for i in 0..32 - ip.cidr {
                bin |= 1 << i;
            }
            bin |= u32::from(x);
            Ip {
                address: Addr::V4(Ipv4Addr::from(bin)),
                cidr: ip.cidr,
            }
        }
        Addr::V6(x) => {
            let mut bin: u128 = 0;
            for i in 0..128 - ip.cidr {
                bin |= 1 << i;
            }
            bin |= u128::from(x);
            Ip {
                address: Addr::V6(Ipv6Addr::from(bin)),
                cidr: ip.cidr,
            }
        }
    }
}

pub fn network(ip: &Ip) -> Ip {
    let mut cidr = ip.cidr;
    match ip.address {
        Addr::V4(x) => {
            if ip.cidr > 32 {
                cidr = 32;
            }

            let mut bin: u32 = 0;
            for i in 0..32 - cidr {
                bin |= 1 << i;
            }
            bin = !(bin) & u32::from(x);
            Ip {
                address: Addr::V4(Ipv4Addr::from(bin)),
                cidr,
            }
        }
        Addr::V6(x) => {
            if ip.cidr > 128 {
                cidr = 128;
            }
            let mut bin: u128 = 0;
            for i in 0..128 - cidr {
                bin |= 1 << i;
            }
            bin = !(bin) & u128::from(x);
            Ip {
                address: Addr::V6(Ipv6Addr::from(bin)),
                cidr,
            }
        }
    }
}

pub fn subnet(ip: &Ip) -> Ip {
    match ip.address {
        Addr::V4(_x) => {
            let mut bin: u32 = 0;
            for i in 0..32 - ip.cidr {
                bin |= 1 << i;
            }
            bin = !bin;
            Ip {
                address: Addr::V4(Ipv4Addr::from(bin)),
                cidr: ip.cidr,
            }
        }
        Addr::V6(_x) => {
            let mut bin: u128 = 0;
            for i in 0..128 - ip.cidr {
                bin |= 1 << i;
            }
            bin = !bin;
            Ip {
                address: Addr::V6(Ipv6Addr::from(bin)),
                cidr: ip.cidr,
            }
        }
    }
}

pub fn wildcard(ip: &Ip) -> Ip {
    match ip.address {
        Addr::V4(_x) => {
            let mut bin: u32 = 0;
            for i in 0..32 - ip.cidr {
                bin |= 1 << i;
            }
            Ip {
                address: Addr::V4(Ipv4Addr::from(bin)),
                cidr: ip.cidr,
            }
        }
        Addr::V6(_x) => {
            let mut bin: u128 = 0;
            for i in 0..128 - ip.cidr {
                bin |= 1 << i;
            }
            Ip {
                address: Addr::V6(Ipv6Addr::from(bin)),
                cidr: ip.cidr,
            }
        }
    }
}

// if addr's network is lower, or addr's broadcast is higher, then not within outside
pub fn within(net: &Ip, addr: &Ip) -> bool {
    match (&net.address, &addr.address) {
        (Addr::V4(_x), Addr::V4(_y)) => {
            let n = network(net);
            let b = broadcast(net);

            let tn = network(addr);
            let tb = broadcast(addr);

            tn.address >= n.address && tb.address <= b.address && net.cidr <= addr.cidr
        }
        (Addr::V6(_x), Addr::V6(_y)) => {
            let n = network(net);
            let b = broadcast(net);

            let tn = network(addr);
            let tb = broadcast(addr);

            tn.address >= n.address && tb.address <= b.address && net.cidr <= addr.cidr
        }
        (_, _) => false,
    }
}

// if addr's network is lower, or addr's broadcast is higher, then not within outside
pub fn without(net: &Ip, addr: &Ip) -> bool {
    match (&net.address, &addr.address) {
        (Addr::V4(_x), Addr::V4(_y)) => {
            let n = network(net);
            let b = broadcast(net);

            let tn = network(addr);
            let tb = broadcast(addr);

            !(tn.address >= n.address && tb.address <= b.address)
        }
        (Addr::V6(_x), Addr::V6(_y)) => {
            let n = network(net);
            let b = broadcast(net);

            let tn = network(addr);
            let tb = broadcast(addr);

            !(tn.address >= n.address && tb.address <= b.address)
        }
        (_, _) => false,
    }
}

// if addr's network is lower, or addr's broadcast is higher, then not within outside
pub fn withoverlap(net: &Ip, addr: &Ip) -> bool {
    match (&net.address, &addr.address) {
        (Addr::V4(_x), Addr::V4(_y)) => {
            let n = network(net);
            let b = broadcast(net);

            let tn = network(addr);
            let tb = broadcast(addr);

            if tn.address <= n.address && tb.address >= n.address {
                return true;
            }

            if tn.address <= b.address && tb.address > b.address {
                return true;
            }

            false
        }
        (Addr::V6(_x), Addr::V6(_y)) => {
            let n = network(net);
            let b = broadcast(net);

            let tn = network(addr);
            let tb = broadcast(addr);

            if tn.address <= n.address && tb.address >= n.address {
                return true;
            }

            if tn.address <= b.address && tb.address > b.address {
                return true;
            }

            false
        }
        (_, _) => false,
    }
}

pub fn network_reservation(ip: &Ip) -> Option<String> {
    let mut rows: HashMap<Ip, String> = HashMap::new();

    rows.insert(
        Ip {
            address: Addr::V4(Ipv4Addr::from_str("0.0.0.0").unwrap()),
            cidr: 8,
        },
        "Current network".to_string(),
    );
    rows.insert( Ip { address: Addr::V4(Ipv4Addr::from_str( "100.64.0.0").unwrap()), cidr: 10}, "Shared address space for communications between a service provider and its subscribers when using a carrier-grade NAT.".to_string() );
    rows.insert(
        Ip {
            address: Addr::V4(Ipv4Addr::from_str("127.0.0.0").unwrap()),
            cidr: 8,
        },
        "Used for loopback addresses to the local host.".to_string(),
    );
    rows.insert( Ip { address: Addr::V4(Ipv4Addr::from_str( "169.254.0.0").unwrap()), cidr: 16}, "Used for link-local addresses between two hosts on a single link when no IP address is otherwise specified, such as would have normally been retrieved from a DHCP server.".to_string() );
    rows.insert(
        Ip {
            address: Addr::V4(Ipv4Addr::from_str("172.16.0.0").unwrap()),
            cidr: 12,
        },
        "Used for local communications within a private network.".to_string(),
    );
    rows.insert(
        Ip {
            address: Addr::V4(Ipv4Addr::from_str("192.0.0.0").unwrap()),
            cidr: 24,
        },
        "IETF Protocol Assignments.".to_string(),
    );
    rows.insert(
        Ip {
            address: Addr::V4(Ipv4Addr::from_str("192.0.2.0").unwrap()),
            cidr: 24,
        },
        "Assigned as TEST-NET-1, documentation and examples.".to_string(),
    );
    rows.insert(
        Ip {
            address: Addr::V4(Ipv4Addr::from_str("192.88.99.0").unwrap()),
            cidr: 24,
        },
        "Reserved. Formerly used for IPv6 to IPv4 relay (included IPv6 address block 2002::/16)."
            .to_string(),
    );
    rows.insert(
        Ip {
            address: Addr::V4(Ipv4Addr::from_str("192.168.0.0").unwrap()),
            cidr: 16,
        },
        "Used for local communications within a private network.".to_string(),
    );
    rows.insert(
        Ip {
            address: Addr::V4(Ipv4Addr::from_str("198.18.0.0").unwrap()),
            cidr: 15,
        },
        "Used for benchmark testing of inter-network communications between two separate subnets."
            .to_string(),
    );
    rows.insert(
        Ip {
            address: Addr::V4(Ipv4Addr::from_str("198.51.100.0").unwrap()),
            cidr: 24,
        },
        "Assigned as TEST-NET-2, documentation and examples.".to_string(),
    );
    rows.insert(
        Ip {
            address: Addr::V4(Ipv4Addr::from_str("203.0.113.0").unwrap()),
            cidr: 24,
        },
        "Assigned as TEST-NET-3, documentation and examples.".to_string(),
    );
    rows.insert(
        Ip {
            address: Addr::V4(Ipv4Addr::from_str("224.0.0.0").unwrap()),
            cidr: 4,
        },
        "In use for IP multicast. (Former Class D network.)".to_string(),
    );
    rows.insert(
        Ip {
            address: Addr::V4(Ipv4Addr::from_str("233.252.0.0").unwrap()),
            cidr: 24,
        },
        "Assigned as MCAST-TEST-NET, documentation and examples.".to_string(),
    );
    rows.insert(
        Ip {
            address: Addr::V4(Ipv4Addr::from_str("240.0.0.0").unwrap()),
            cidr: 4,
        },
        "Reserved for future use. (Former Class E network.)".to_string(),
    );
    rows.insert(
        Ip {
            address: Addr::V4(Ipv4Addr::from_str("255.255.255.255").unwrap()),
            cidr: 32,
        },
        "Reserved for limited broadcast destination address.".to_string(),
    );

    rows.insert(
        Ip {
            address: Addr::V4(Ipv4Addr::from_str("10.0.0.0").unwrap()),
            cidr: 8,
        },
        "RFC 1918".to_string(),
    );
    rows.insert(
        Ip {
            address: Addr::V4(Ipv4Addr::from_str("172.16.0.0").unwrap()),
            cidr: 12,
        },
        "RFC 1918".to_string(),
    );
    rows.insert(
        Ip {
            address: Addr::V4(Ipv4Addr::from_str("192.168.0.0").unwrap()),
            cidr: 16,
        },
        "RFC 1918".to_string(),
    );

    rows.insert(
        Ip {
            address: Addr::V6(Ipv6Addr::from_str("::1").unwrap()),
            cidr: 128,
        },
        "Loopback address".to_string(),
    );
    rows.insert(
        Ip {
            address: Addr::V6(Ipv6Addr::from_str("::ffff:0:0").unwrap()),
            cidr: 96,
        },
        "IPv4-mapped addresses".to_string(),
    );
    rows.insert(
        Ip {
            address: Addr::V6(Ipv6Addr::from_str("::ffff:0:0:0").unwrap()),
            cidr: 96,
        },
        "IPv4 translated addresses".to_string(),
    );
    rows.insert(
        Ip {
            address: Addr::V6(Ipv6Addr::from_str("64:ff9b::").unwrap()),
            cidr: 96,
        },
        "IPv4/IPv6 translation".to_string(),
    );
    rows.insert(
        Ip {
            address: Addr::V6(Ipv6Addr::from_str("64:ff9b:1::").unwrap()),
            cidr: 48,
        },
        "IPv4/IPv6 translation".to_string(),
    );
    rows.insert(
        Ip {
            address: Addr::V6(Ipv6Addr::from_str("100::").unwrap()),
            cidr: 64,
        },
        "Discard prefix".to_string(),
    );
    rows.insert(
        Ip {
            address: Addr::V6(Ipv6Addr::from_str("2001:0000::").unwrap()),
            cidr: 32,
        },
        "Teredo tunneling".to_string(),
    );
    rows.insert(
        Ip {
            address: Addr::V6(Ipv6Addr::from_str("2001:20::").unwrap()),
            cidr: 28,
        },
        "ORCHIDv2".to_string(),
    );
    rows.insert(
        Ip {
            address: Addr::V6(Ipv6Addr::from_str("2001:db8::").unwrap()),
            cidr: 32,
        },
        "Documentation range".to_string(),
    );
    rows.insert(
        Ip {
            address: Addr::V6(Ipv6Addr::from_str("2002::").unwrap()),
            cidr: 16,
        },
        "The 6to4 addressing scheme (legacy)".to_string(),
    );
    rows.insert(
        Ip {
            address: Addr::V6(Ipv6Addr::from_str("fc00::").unwrap()),
            cidr: 7,
        },
        "Unique local address".to_string(),
    );
    rows.insert(
        Ip {
            address: Addr::V6(Ipv6Addr::from_str("ff00::").unwrap()),
            cidr: 8,
        },
        "Multicast address".to_string(),
    );
    rows.insert(
        Ip {
            address: Addr::V6(Ipv6Addr::from_str("fe80::").unwrap()),
            cidr: 10,
        },
        "Link local".to_string(),
    );
    rows.insert(
        Ip {
            address: Addr::V6(Ipv6Addr::from_str("fd00::").unwrap()),
            cidr: 8,
        },
        "Link local".to_string(),
    );

    for i in network_iter(&ip.clone()) {
        let net_row = rows.get(&network(&i));

        if let Some(s) = net_row {
            return Some(s.to_string());
        }
    }

    None
}

pub fn matching_network_interface(
    ip: &Ip,
    interfaces: &[InterfaceAddress],
    ip_only: bool,
) -> String {
    let net = network(ip);
    for ifaddr in interfaces {
        if let Some(address) = ifaddr.address {
            // println!("interface {} address {}", ifaddr.interface_name, address);
            let ss: SockaddrStorage = address;

            match address.family() {
                Some(AddressFamily::Inet) => {
                    let if_addr_bin = ss.as_sockaddr_in().unwrap().ip();

                    if ip_only {
                        if let Addr::V4(x) = ip.address {
                            if u32::from(if_addr_bin) == u32::from(x) {
                                return ifaddr.interface_name.to_string();
                            }
                        }
                        continue;
                    }

                    if let Some(netmask) = ifaddr.netmask {
                        if let Addr::V4(x) = net.address {
                            let bin = u32::from(x);
                            let a = netmask.as_sockaddr_in().unwrap().ip();

                            if (bin & u32::from(a)) == (u32::from(if_addr_bin) & u32::from(a)) {
                                //println!("{:#?} in {} {}", ip, bin, a);
                                return ifaddr.interface_name.to_string();
                            }
                        }
                    }
                }
                Some(AddressFamily::Inet6) => {
                    let if_addr_bin = u128::from(ss.as_sockaddr_in6().unwrap().ip());

                    if ip_only {
                        if let Addr::V6(x) = ip.address {
                            if if_addr_bin == u128::from(x) {
                                return ifaddr.interface_name.to_string();
                            }
                        }
                        continue;
                    }

                    if let Some(netmask) = ifaddr.netmask {
                        if let Addr::V6(x) = net.address {
                            let bin = u128::from(x);
                            let a = u128::from(netmask.as_sockaddr_in6().unwrap().ip());
                            if (bin & a) == (if_addr_bin & a) {
                                // println!("{:#?} block {} {}", ip, bin, a);
                                return ifaddr.interface_name.to_string();
                            }
                        }
                    }
                }
                _ => {
                    continue;
                }
            }
        }
    }
    "".to_string()
}

pub fn rbl_format(ip: &Ip) -> String {
    match ip.address {
        Addr::V4(x) => {
            let f = format!("{}", x);
            let v: Vec<&str> = f.rsplit('.').collect();
            v.join(".")
        }
        Addr::V6(x) => {
            let mut v = vec![];
            let octet = x.octets();
            for o in octet.iter().rev() {
                let r = format!("{:02x}", o);

                v.push(r.get(1..2).unwrap().to_string());
                v.push(r.get(0..1).unwrap().to_string());
            }
            v.join(".")
        }
    }
}

pub fn ip_lookup(address: &str, hm: &mut HashMap<String, String>) -> String {
    let k = format!("n/{}", address);

    if let Some(v) = hm.get(&k) {
        return v.clone();
    }

    if let Ok(buffer) = lookup_host(address) {
        let arg = buffer[0].to_string();

        hm.insert(k, arg.clone());
        arg
    } else {
        "".to_string()
    }
}

pub fn ptr_format(ip: &Ip, hm: &mut HashMap<String, String>) -> String {
    let k: String = ip.to_string();

    if let Some(v) = hm.get(&k) {
        return v.clone();
    }

    let j = match ip.address {
        Addr::V4(x) => {
            let ip: std::net::IpAddr = std::net::IpAddr::V4(x);
            lookup_addr(&ip)
        }
        Addr::V6(x) => {
            let ip: std::net::IpAddr = std::net::IpAddr::V6(x);
            lookup_addr(&ip)
        }
    };

    if let Ok(j) = j {
        hm.insert(k, j.clone());

        return j;
    }

    "".to_string()
}

pub fn network_size(ip: &Ip) -> u128 {
    let start = network(ip);
    let end = broadcast(ip);

    match (start.address, end.address) {
        (Addr::V6(start), Addr::V6(end)) => return (u128::from(end) - u128::from(start)) + 1,
        (Addr::V4(start), Addr::V4(end)) => return (u32::from(end) - u32::from(start)) as u128 + 1,
        (_, _) => {}
    }
    1
}

pub fn formatted_address(ip: &Ip, mode: &FormatMode) -> String {
    match mode {
        FormatMode::Text => ip.to_string(),
        FormatMode::Integer => ip.num_representation(),
        FormatMode::SignedInteger => ip.signed_num_representation(),
        FormatMode::Hex => ip.hex_quad_representation(),
        FormatMode::SplitBinary => ip.bin_split_representation(),
        FormatMode::Binary => ip.bin_representation(),
        FormatMode::Backslash => "".to_string(),
    }
}

pub fn cdb_lookup(ip: &mut Ip, config: &RefCell<Config>) -> Option<HashMap<String, String>> {
    let mut hm: HashMap<String, String> = HashMap::new();

    let mut config = config.borrow_mut();

    if config.cdb.is_none() {
        let path = config.options.get("cdb_path");

        path?;

        let path = path.unwrap();
        let cdb = cdb::CDB::open(path);
        if cdb.is_err() {
            eprintln!("cannot open {}", path);
            return None;
        }

        config.cdb = Some(cdb.unwrap().into());
    };

    let lookup_ip = ip.clone();

    for k in network_iter(&lookup_ip) {
        let str_key = format!("{}/{}", network(&k), k.cidr);
        let key = str_key.as_bytes();
        let result = config.cdb.as_ref().unwrap().get(key);
        if result.is_none() {
            continue;
        }

        *ip = k;

        let result = result.unwrap().unwrap();
        if result.is_empty() {
            return None;
        }

        let value = String::from_utf8(result).unwrap();
        let value: Vec<&str> = value.split('\0').collect();
        for v in value {
            let p: Vec<&str> = v.split('=').collect();
            if p.len() < 2 {
                continue;
            }

            hm.insert(p[0].to_string(), p[1].to_string());
        }

        return Some(hm);
    }

    None
}

pub fn config_option_true(config: &Config, opt: String) -> bool {
    config.options.get(&opt).unwrap_or(&"false".to_string()) != "false"
}

pub fn geoip2_city(config: &RefCell<Config>, ip: &Ip) -> Option<HashMap<String, String>> {
    let mut ret: HashMap<String, String> = HashMap::new();

    if config.borrow().mmcity.is_none() {
        let path = config.borrow().options.get("mmcity").unwrap().clone();
        config.borrow_mut().mmcity = Some(
            maxminddb::Reader::open_readfile(path).unwrap().into()
        );
    }

    let binding = config.borrow();
    let reader = binding.mmcity.as_ref().unwrap();

    let f: IpAddr = ip.to_string().parse().unwrap();
    let city: Option<geoip2::City> = reader.lookup(f).unwrap();

    if let Some(city) = city {
        if city.city.is_none() {
            return Some(ret);
        }

        let config = config.borrow();
        let default_lang = "en".to_string();
        let lang = config.options.get("mmlang").unwrap_or(&default_lang);
        let c = city
            .city
            .as_ref()
            .unwrap()
            .names
            .as_ref()
            .unwrap()
            .get(lang as &str);
        if let Some(c) = c {
            ret.insert("city".to_string(), c.to_string());
        }

        let c = city
            .continent
            .as_ref()
            .unwrap()
            .names
            .as_ref()
            .unwrap()
            .get(lang as &str);
        if let Some(c) = c {
            ret.insert("continent".to_string(), c.to_string());
        }

        let c = city
            .country
            .as_ref()
            .unwrap()
            .names
            .as_ref()
            .unwrap()
            .get(lang as &str);
        if let Some(c) = c {
            ret.insert("country".to_string(), c.to_string());
        }

        let c = city
            .country
            .as_ref()
            .unwrap()
            .iso_code;
        if let Some(c) = c {
            ret.insert("code".to_string(), c.to_string());
        }

        let c = city.location.as_ref().unwrap().latitude;
        if let Some(c) = c {
            ret.insert("latitude".to_string(), c.to_string());
        }

        let c = city.location.as_ref().unwrap().longitude;
        if let Some(c) = c {
            ret.insert("longitude".to_string(), c.to_string());
        }
    }

    Some(ret)
}

pub fn geoip2_asn(config: &RefCell<Config>, ip: &Ip) -> Option<HashMap<String, String>> {
    let mut ret: HashMap<String, String> = HashMap::new();

    if config.borrow().mmasn.is_none() {
        let path = config.borrow().options.get("mmasn").unwrap().clone();
        config.borrow_mut().mmasn = Some(
            maxminddb::Reader::open_readfile( path ).unwrap().into()
        );
    }

    let binding = config.borrow();
    let reader = binding.mmasn.as_ref().unwrap();

    let f: IpAddr = ip.to_string().parse().unwrap();
    let asn: Option<geoip2::Asn> = reader.lookup(f).unwrap();

    if let Some(asn) = asn {
        let c = asn.autonomous_system_number.as_ref().unwrap();
        ret.insert("autonomous_system_number".to_string(), c.to_string());

        let c = asn.autonomous_system_organization.as_ref().unwrap();
        ret.insert("autonomous_system_organization".to_string(), c.to_string());
    }

    Some(ret)
}

pub fn geoip2_country(config: &RefCell<Config>, ip: &Ip) -> Option<HashMap<String, String>> {
    let mut ret: HashMap<String, String> = HashMap::new();

    if config.borrow().mmcountry.is_none() {
        let path = config.borrow().options.get("mmcountry").unwrap().clone();
        config.borrow_mut().mmcountry = Some(
            maxminddb::Reader::open_readfile(path).unwrap().into()
        );
    }

    let binding = config.borrow();
    let reader = binding.mmcountry.as_ref().unwrap();

    let f: IpAddr = ip.to_string().parse().unwrap();
    let country: Option<geoip2::Country> = reader.lookup(f).unwrap();

    if let Some(country) = country {
        let config = config.borrow();
        let default_lang = "en".to_string();
        let lang = config.options.get("mmlang").unwrap_or(&default_lang);

        let c = country
            .continent
            .as_ref()
            .unwrap()
            .names
            .as_ref()
            .unwrap()
            .get(lang as &str);
        if let Some(c) = c {
            ret.insert("continent".to_string(), c.to_string());
        }

        let c = country
            .country
            .as_ref()
            .unwrap()
            .names
            .as_ref()
            .unwrap()
            .get(lang as &str);
        if let Some(c) = c {
            ret.insert("country".to_string(), c.to_string());
        }

        let c = country
            .registered_country
            .as_ref()
            .unwrap()
            .names
            .as_ref()
            .unwrap()
            .get(lang as &str);
        if let Some(c) = c {
            ret.insert("registered_country".to_string(), c.to_string());
        }
    }

    Some(ret)
}

pub fn format_details(
    format_detail: &FormatDetail,
    formatted: String,
    rows: &Option<HashMap<Ip, NetRow>>,
    subnet_size: Option<u32>,
    matches: Option<&getopts::Matches>,
    config: &RefCell<Config>,
) -> Option<String> {
    let ip = &mut (*format_detail.ip.as_ref().unwrap()).clone();
    let mut reformatted = formatted;

    if rows.is_some() {
        let mut found_match = false;

        let rows = rows.as_ref().unwrap();

        for i in network_iter(&Ip {
            address: ip.address.clone(),
            cidr: ip.cidr,
        }) {
            let net_row = rows.get(&i);

            if net_row.is_none() {
                continue;
            }

            let net_row = net_row.unwrap();

            for f in net_row.row.keys() {
                reformatted =
                    reformatted.replace(&format!("%{{{}}}", f), net_row.row.get(f).unwrap());
            }

            ip.cidr = i.cidr;
            found_match = true;

            break;
        }
        if !found_match {
            if let Some(m) = matches {
                if !m.opt_present("allowemptyrow") {
                    return None;
                }
            };
        }
    }

    if config_option_true(&config.borrow(), "abuseipdb".to_string())
        && reformatted.contains("%{abuseipdb")
    {
        let key = config
            .borrow()
            .options
            .get("abuseipdb")
            .unwrap()
            .to_string();
        let map = get_abuseipdb_details(config, ip, &key);

        for k in map.keys() {
            let word = format!("abuseipdb_{}", k);
            reformatted = reformatted.replace(&format!("%{{{}}}", word), map.get(k).unwrap());
        }
    }

    // need to make this more like the loop above
    if config.borrow().options.contains_key("cdb_path") {
        let mut lookup_ip = ip.clone();
        if let Some(row) = cdb_lookup(&mut lookup_ip, config) {
            for f in row.keys() {
                reformatted = reformatted.replace(&format!("%{{{}}}", f), row.get(f).unwrap());
            }

            ip.cidr = lookup_ip.cidr;
        } else if let Some(m) = matches {
            if !m.opt_present("allowemptyrow") {
                return None;
            }
        }
    }

    if (config_option_true(&config.borrow(), "mmasn".to_string())
        || config_option_true(&config.borrow(), "mmcity".to_string())
        || config_option_true(&config.borrow(), "mmcountry".to_string()))
        && (reformatted.contains("%{mmcity")
            || reformatted.contains("%{mmcountry")
            || reformatted.contains("%{mmasn"))
    {
        if reformatted.contains("%{mmasn") {
            let map = geoip2_asn(config, ip);
            if let Some(map) = map {
                for k in map.keys() {
                    let word = format!("mmasn_{}", k);
                    reformatted =
                        reformatted.replace(&format!("%{{{}}}", word), map.get(k).unwrap());
                }
            }
        }

        if reformatted.contains("%{mmcity") {
            let map = geoip2_city(config, ip);
            if let Some(map) = map {
                for k in map.keys() {
                    let word = format!("mmcity_{}", k);
                    reformatted =
                        reformatted.replace(&format!("%{{{}}}", word), map.get(k).unwrap());
                }
            }
        }

        if reformatted.contains("%{mmcountry") {
            let map = geoip2_country(config, ip);
            if let Some(map) = map {
                for k in map.keys() {
                    let word = format!("mmcountry_{}", k);
                    reformatted =
                        reformatted.replace(&format!("%{{{}}}", word), map.get(k).unwrap());
                }
            }
        }
    }

    if let Some(line) = &format_detail.line {
        reformatted = reformatted.replace("%{line}", line);
    }

    let b = broadcast(ip);
    let n = network(ip);
    let s = subnet(ip);
    let w = wildcard(ip);

    if let Some(r) = network_reservation(ip) {
        reformatted = reformatted.replace("%r", &r);
    }

    let mut mode = FormatMode::Text;
    let mut out_str = "".to_string();
    let chars: Vec<_> = reformatted.chars().collect();

    let mut format_processor = FormatProcessor::None;
    for k in chars {
        match format_processor {
            FormatProcessor::Percent => {
                format_processor = FormatProcessor::None;
                match k {
                    'B' => {
                        format_processor = FormatProcessor::Percent;
                        mode = FormatMode::Binary;
                    }
                    'S' => {
                        format_processor = FormatProcessor::Percent;
                        mode = FormatMode::SplitBinary;
                    }
                    'l' => {
                        format_processor = FormatProcessor::Percent;
                        mode = FormatMode::Integer;
                    }
                    'L' => {
                        format_processor = FormatProcessor::Percent;
                        mode = FormatMode::SignedInteger;
                    }
                    'x' => {
                        format_processor = FormatProcessor::Percent;
                        mode = FormatMode::Hex;
                    }
                    'a' => {
                        out_str.push_str(&formatted_address(ip, &mode));
                    }
                    'b' => {
                        out_str.push_str(&formatted_address(&b, &mode));
                    }
                    'n' => {
                        out_str.push_str(&formatted_address(&n, &mode));
                    }
                    'w' => {
                        out_str.push_str(&formatted_address(&w, &mode));
                    }
                    's' => {
                        out_str.push_str(&formatted_address(&s, &mode));
                    }
                    'c' => {
                        out_str.push_str(&ip.cidr.to_string());
                    }
                    't' => {
                        out_str.push_str(&network_size(ip).to_string());
                    }
                    'm' => {
                        out_str.push_str(&matching_network_interface(
                            ip,
                            &config.borrow().interface_names,
                            false,
                        ));
                    }
                    'd' => {
                        out_str.push_str(&matching_network_interface(
                            ip,
                            &config.borrow().interface_names,
                            true,
                        ));
                    }
                    'k' => {
                        out_str.push_str(&rbl_format(ip));
                    }
                    'p' => {
                        out_str.push_str(&ptr_format(ip, &mut config.borrow_mut().hm));
                    }
                    '%' => {
                        out_str.push('%');
                    }
                    'D' => {
                        if let Some(s) = subnet_size {
                            out_str.push_str(&s.to_string());
                        } else {
                            out_str.push('D');
                        };
                    }
                    'N' => {
                        if let Some(s) = subnet_size {
                            out_str.push_str(&subnets_in_network(s, ip).to_string());
                        } else {
                            out_str.push('N');
                        };
                    }
                    'C' => {
                        let config = config.borrow();
                        if config_option_true(&config, "countseen".to_string())
                            && config.count.is_some()
                        {
                            out_str.push_str(&get_seen_count(&config, ip).to_string());
                        } else if let Some(used) = &config.used {
                            let count = used.keys().count();
                            out_str.push_str(&count.to_string());
                        } else {
                            out_str.push('C');
                        }
                    }
                    _ => {
                        out_str.push(k);
                    }
                }
                continue;
            }
            FormatProcessor::Backslash => {
                format_processor = FormatProcessor::None;
                match k {
                    'n' => {
                        out_str.push('\n');
                        mode = FormatMode::Text;
                    }
                    't' => {
                        out_str.push('\t');
                        mode = FormatMode::Text;
                    }
                    '\\' => {
                        out_str.push('\\');
                        mode = FormatMode::Text;
                    }
                    _ => {
                        out_str.push(k);
                        mode = FormatMode::Text;
                    }
                }
                continue;
            }
            FormatProcessor::None => {}
        }

        match k {
            '%' => {
                format_processor = FormatProcessor::Percent;
                mode = FormatMode::Text;
            }
            '\\' => {
                format_processor = FormatProcessor::Backslash;
                mode = FormatMode::Text;
            }
            _ => {
                out_str.push(k);
            }
        }
    }

    match format_processor {
        FormatProcessor::Percent => out_str.push('%'),
        FormatProcessor::Backslash => out_str.push('\\'),
        FormatProcessor::None => {}
    }

    Some(out_str)
}

pub fn fd_ready(fd: BorrowedFd) -> bool {
    let s = fstat(fd);
    if let Ok(x) = s {
        if SFlag::S_IFIFO.bits() & x.st_mode == SFlag::S_IFIFO.bits() {
            return true;
        }
    }

    false
}

pub fn find_ips<'a>(
    reader: &'a mut Box<dyn BufRead>,
    input_base: Option<i32>,
    reverse: &'a Reverse,
    config: &'a RefCell<Config>,
) -> impl 'a + std::iter::Iterator<Item = Vec<Ip>> {
    std::iter::from_fn(move || {
        if let Some(line) = reader.lines().next().into_iter().by_ref().next() {
            let line: String = line.as_ref().unwrap().trim().to_string();
            let mut v = vec![];

            for part in line.split(' ') {
                let p = part.trim();
                if p.is_empty() {
                    continue;
                }

                let ip = match parse_address_mask(
                    p,
                    Some(32),
                    Some(128),
                    input_base,
                    matches!(reverse, Reverse::Both | Reverse::Input),
                    config,
                ) {
                    Some(x) => x,
                    None => {
                        if config
                            .borrow()
                            .options
                            .get("quiet")
                            .unwrap_or(&"false".to_string())
                            != "true"
                        {
                            eprintln!("Could not parse {}", p);
                        }
                        continue;
                    }
                };

                v.push(ip);
            }
            return Some(v);
        }
        None
    })
}

pub fn subnets_in_network(networks: u32, ip: &Ip) -> u128 {
    let base: u128 = 2;
    base.pow(networks - ip.cidr)
}

pub fn is_like_hostname(word: &str) -> bool {
    let r = Regex::new(r#"^(\S+\.)?(xn--)?[a-z0-9][a-z0-9-_]{0,61}[a-z0-9]{0,1}\.(xn--)?([a-z0-9\-]{1,61}|[a-z0-9-]{1,30}\.[a-z]{2,})$"#).unwrap();

    r.is_match(word)
}

pub fn inside_filter(inside: Option<bool>, ip_args: &[Ip], ip: &Ip) -> bool {
    match inside {
        Some(true) => {
            let mut found = false;
            for arg in ip_args {
                if within(arg, ip) {
                    found = true;
                    break;
                }
            }

            if found {
                return true;
            }
        }
        Some(false) => {
            let mut found = false;

            for arg in ip_args {
                if within(arg, ip) {
                    found = true;
                    break;
                }
            }

            if !found {
                return true;
            }
        }
        None => {
            return true;
        }
    }

    false
}

pub fn group_mask(config: &Config, i: &Ip) -> u32 {
    match i.address {
        Addr::V4(_) => {
            if config_option_true(config, "group4".to_string()) {
                return config
                    .options
                    .get("group4")
                    .expect("cannot read group4")
                    .parse::<u32>()
                    .unwrap();
            }
        }
        Addr::V6(_) => {
            if config_option_true(config, "group6".to_string()) {
                return config
                    .options
                    .get("group6")
                    .expect("cannot read group6")
                    .parse::<u32>()
                    .unwrap();
            }
        }
    }

    if config_option_true(config, "group".to_string()) {
        return config
            .options
            .get("group")
            .expect("cannot read group")
            .parse::<u32>()
            .unwrap();
    }

    128
}

pub fn config_cidr_grouping(config: &Config) -> bool {
    config_option_true(config, "group".to_string())
        || config_option_true(config, "group4".to_string())
        || config_option_true(config, "group6".to_string())
}

pub fn increment_seen_count(config: &mut Config, ip: Ip) {
    if config_option_true(config, "countseen".to_string()) && config.count.is_some() {
        let cidr = group_mask(config, &ip);

        let net_mask = network(&Ip {
            address: ip.address.clone(),
            cidr,
        });

        let count = config.count.as_ref().unwrap().get(&net_mask).unwrap_or(&0) + 1;
        config.count.as_mut().unwrap().insert(net_mask, count);
    }
}

pub fn get_seen_count(config: &Config, i: &Ip) -> usize {
    if config_option_true(config, "countseen".to_string()) && config.count.is_some() {
        let net_mask = network(&Ip {
            address: i.address.clone(),
            cidr: config
                .options
                .get("group")
                .unwrap_or(&"128".to_string())
                .parse()
                .unwrap(),
        });

        return *config.count.as_ref().unwrap().get(&net_mask).unwrap_or(&0);
    }
    0
}

pub fn ua_client() -> Option<reqwest::blocking::Client> {
    reqwest::blocking::Client::builder()
        .user_agent(format!(
            "{}/{}",
            env!("CARGO_PKG_NAME"),
            env!("CARGO_PKG_VERSION")
        ))
        .build()
        .ok()
}

pub fn get_abuseipdb_details(
    config: &RefCell<Config>,
    ip: &Ip,
    key: &str,
) -> HashMap<String, String> {
    if config.borrow().abuse_cache.is_none() {
        {
            config.borrow_mut().abuse_cache = Some(HashMap::new());
        }
    }

    let body = {
        // let abuse_cache = &config.borrow().abuse_cache;
        if config
            .borrow()
            .abuse_cache
            .as_ref()
            .unwrap()
            .get(ip)
            .is_some()
        {
            config
                .borrow()
                .abuse_cache
                .as_ref()
                .unwrap()
                .get(ip)
                .unwrap()
                .to_string()
        } else {
            let mut headers = HeaderMap::new();
            headers.insert("Key", key.parse().unwrap());

            let client = ua_client().expect("Cannot make user-agent");
            let response = client
                .get(format!(
                    "https://api.abuseipdb.com/api/v2/check?ipAddress={}&maxAgeInDays={}",
                    ip, 30
                ))
                .header("Key".to_string(), key)
                .send();

            if response.as_ref().is_err() {
                eprintln!("Could not communicate with abuseipdb.com");
                std::process::exit(1);
            }

            let resp = response.unwrap().text().unwrap();
            config
                .borrow_mut()
                .abuse_cache
                .as_mut()
                .unwrap()
                .insert(ip.clone(), resp.clone());
            resp
        }
    };

    let h: Value = serde_json::from_str(&body).unwrap();

    let mut config = config.borrow_mut();
    if h["data"].as_object().is_none() {
        eprintln!("Error communicating with abuseipdb.com");
        std::process::exit(1);
    }

    let data = h["data"].as_object().unwrap();
    let mut map: HashMap<String, String> = HashMap::new();
    for k in data.keys() {
        config
            .options
            .insert(format!("abuseipdb_{}", k).to_string(), data[k].to_string());

        map.insert(k.to_string(), data[k].to_string());
    }

    map
}

pub fn line_filter(
    config: &RefCell<Config>,
    position: Option<u32>,
    line: &str,
    base: Option<i32>,
) -> Option<Ip> {
    let parts: Vec<&str> = line.split(' ').collect();

    if parts.is_empty() {
        return None;
    }

    if let Some(x) = position {
        if parts.len() < (x + 1) as usize {
            return None;
        }

        if let Some(ip) =
            parse_address_mask(parts[x as usize].trim(), None, None, base, false, config)
        {
            return Some(ip);
        }
    }

    if position.is_none() {
        for p in &parts {
            let p = p.trim();

            if p.is_empty() {
                continue;
            }

            if let Some(ip) = parse_address_mask(p, None, None, base, false, config) {
                return Some(ip);
            }
        }
    }

    None
}
