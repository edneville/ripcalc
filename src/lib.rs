use nix::ifaddrs::*;
use nix::sys::socket::AddressFamily;
use nix::sys::socket::SockaddrLike;
use nix::sys::socket::SockaddrStorage;
use nix::sys::stat::fstat;
use nix::sys::stat::SFlag;
use std::collections::HashMap;
use std::fmt;
use std::net::Ipv4Addr;
use std::net::Ipv6Addr;
use std::net::ToSocketAddrs;
use std::os::unix::io::RawFd;
use std::str::FromStr;
use std::io::BufRead;

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

pub enum Reverse {
    None,
    Input,
    Source,
    Both,
}

impl Ip {
    fn num_representation(&self) -> String {
        match self.address {
            Addr::V4(x) => u32::from(x).to_string(),
            Addr::V6(x) => u128::from(x).to_string(),
        }
    }
    fn signed_num_representation(&self) -> String {
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
    fn bin_representation(&self) -> String {
        match self.address {
            Addr::V4(x) => format!("{:032b}", u32::from(x)),
            Addr::V6(x) => format!("{:0128b}", u128::from(x)),
        }
    }
    fn bin_split_representation(&self) -> String {
        let mut s = match self.address {
            Addr::V4(x) => format!("{:032b}", u32::from(x)),
            Addr::V6(x) => format!("{:0128b}", u128::from(x)),
        };
        s.insert(self.cidr as usize, ' ');
        s
    }
    fn hex_quad_representation(&self) -> String {
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

pub fn parse_mask(mask: &str) -> Option<u32> {
    match mask.parse::<u32>() {
        Ok(n) => Some(n),
        Err(_) => None,
    }
}

pub fn parse_v6(address: &str, input_base: Option<i32>, reverse: bool) -> Option<Addr> {
    match input_base {
        Some(base) => Some(Addr::V6(Ipv6Addr::from(
            match u128::from_str_radix(address, base as u32) {
                Ok(y) => y,
                Err(e) => {
                    eprintln!("cannot convert {}: {}", address, e);
                    return None;
                }
            },
        ))),
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
                return Some(Addr::V4(Ipv4Addr::from(
                    match u32::from_str_radix(address, base as u32) {
                        Ok(y) => y,
                        Err(e) => {
                            eprintln!("cannot convert {}: {}", address, e);
                            return None;
                        }
                    },
                )));
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
) -> Option<Ip> {
    let parts: Vec<&str> = a.split('/').collect();

    let mut arg = parts[0];

    let mut input_mask: Option<u32> = None;
    if parts.len() > 1 {
        if let Some(m) = parse_mask(parts[1]) {
            input_mask = Some(m);
        }
    };

    let input_ip = parse_v4_v6(arg, input_base, reverse);

    if let Some(input_ip) = input_ip {
        return Some(Ip {
            address: input_ip.clone(),
            cidr: match input_ip {
                Addr::V4(_) => input_mask.unwrap_or_else(|| default_v4_mask.unwrap_or(24)),
                Addr::V6(_) => input_mask.unwrap_or_else(|| default_v6_mask.unwrap_or(64)),
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

    let addrs_iter = format!("{}:443", arg).to_socket_addrs();
    let mut buffer: String;

    if let Ok(mut address) = addrs_iter {
        buffer = format!("{}", address.next().unwrap());
        let v: Vec<&str> = buffer.split(':').collect();
        buffer = v[0].to_string();
        arg = buffer.as_str();
    }

    let input_ip = parse_v4_v6(arg, input_base, reverse);

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

pub fn addresses<'a>(
    ip: &'a Ip,
    used: Option<&'a HashMap<Addr, bool>>,
    mask: Option<u32>,
) -> impl std::iter::Iterator<Item = Ip> + 'a {
    let b = broadcast(ip);
    let mut net = network(ip);

    std::iter::from_fn(move || {
        if let Addr::V4(mut x) = net.address {
            if let Addr::V4(y) = b.address {
                let adder = match mask {
                    Some(x) => {
                        net.cidr = x;
                        let base: u32 = 2;
                        base.pow(32 - x)
                    }
                    None => 1,
                };
                while u32::from(x) <= u32::from(y) {
                    net = Ip {
                        address: Addr::V4(Ipv4Addr::from(u32::from(x) + adder)),
                        cidr: net.cidr,
                    };

                    if let Addr::V4(a) = net.address {
                        x = a
                    };

                    match &used {
                        Some(map) => {
                            if map
                                .get(&Addr::V4(Ipv4Addr::from(u32::from(x) - 1)))
                                .is_some()
                            {
                                continue;
                            }
                        }
                        None => {}
                    }

                    return Some(if mask.is_none() {
                        Ip {
                            address: Addr::V4(Ipv4Addr::from(u32::from(x) - 1)),
                            cidr: net.cidr,
                        }
                    } else {
                        network(&Ip {
                            address: Addr::V4(Ipv4Addr::from(u32::from(x) - 1)),
                            cidr: net.cidr,
                        })
                    });
                }
            }
        }

        if let Addr::V6(mut x) = net.address {
            if let Addr::V6(y) = b.address {
                let adder = match mask {
                    Some(x) => {
                        net.cidr = x;
                        let base: u128 = 2;
                        base.pow(128 - x)
                    }
                    None => 1,
                };

                while u128::from(x) < u128::from(y) {
                    net = Ip {
                        address: Addr::V6(Ipv6Addr::from(u128::from(x) + adder)),
                        cidr: net.cidr,
                    };

                    if let Addr::V6(a) = net.address {
                        x = a
                    };

                    match &used {
                        Some(map) => {
                            if map
                                .get(&Addr::V6(Ipv6Addr::from(u128::from(x) - 1)))
                                .is_some()
                            {
                                continue;
                            }
                        }
                        None => {}
                    }

                    return Some(if mask.is_none() {
                        Ip {
                            address: Addr::V6(Ipv6Addr::from(u128::from(x) - 1)),
                            cidr: net.cidr,
                        }
                    } else {
                        network(&Ip {
                            address: Addr::V6(Ipv6Addr::from(u128::from(x) - 1)),
                            cidr: net.cidr,
                        })
                    });
                }
            }
        }

        None
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
            (Addr::V4(_), Addr::V4(_)) => {
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
            (Addr::V6(_), Addr::V6(_)) => {
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
    match ip.address {
        Addr::V4(x) => {
            let mut bin: u32 = 0;
            for i in 0..32 - ip.cidr {
                bin |= 1 << i;
            }
            bin = !(bin) & u32::from(x);
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
            bin = !(bin) & u128::from(x);
            Ip {
                address: Addr::V6(Ipv6Addr::from(bin)),
                cidr: ip.cidr,
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

    match ip.address {
        Addr::V4(x) => {
            let search_ip = x;

            for i in 0..32 {
                let cidr = 32 - i;

                let net_row = rows.get(&network(&Ip {
                    address: Addr::V4(search_ip),
                    cidr,
                }));

                if let Some(s) = net_row {
                    return Some(s.to_string());
                }
            }
        }
        Addr::V6(x) => {
            let search_ip = x;

            for i in 0..128 {
                let cidr = 128 - i;

                let net_row = rows.get(&network(&Ip {
                    address: Addr::V6(search_ip),
                    cidr,
                }));

                if let Some(s) = net_row {
                    return Some(s.to_string());
                }
            }
        }
    }

    None
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

pub fn format_details(
    ip: &Ip,
    formatted: String,
    rows: &Option<HashMap<Ip, NetRow>>,
) -> Option<String> {
    let ip = &mut ip.clone();
    let mut reformatted = formatted;

    if rows.is_some() {
        let mut found_match = false;
        let rows = rows.as_ref().unwrap();

        match ip.address {
            Addr::V4(x) => {
                let search_ip = x;

                for i in 0..32 {
                    let cidr = 32 - i;
                    let mn = network(&Ip {
                        address: Addr::V4(search_ip),
                        cidr,
                    });

                    let net_row = rows.get(&Ip {
                        address: mn.address,
                        cidr,
                    });

                    if net_row.is_none() {
                        continue;
                    }

                    let net_row = net_row.unwrap();
                    ip.cidr = cidr;

                    for f in net_row.row.keys() {
                        reformatted = reformatted
                            .replace(&format!("%{{{}}}", f), net_row.row.get(f).unwrap());
                    }
                    found_match = true;

                    break;
                }
            }
            Addr::V6(x) => {
                let search_ip = x;

                for i in 0..128 {
                    let cidr = 128 - i;
                    let mn = network(&Ip {
                        address: Addr::V6(search_ip),
                        cidr,
                    });

                    let net_row = rows.get(&Ip {
                        address: mn.address,
                        cidr,
                    });

                    if net_row.is_none() {
                        continue;
                    }

                    ip.cidr = cidr;

                    let net_row = net_row.unwrap();

                    for f in net_row.row.keys() {
                        reformatted = reformatted
                            .replace(&format!("%{{{}}}", f), net_row.row.get(f).unwrap());
                    }
                    found_match = true;

                    break;
                }
            }
        }
        if !found_match {
            return None;
        }
    }
    let b = broadcast(ip);
    let n = network(ip);
    let s = subnet(ip);
    let w = wildcard(ip);

    if let Some(r) = network_reservation(ip) {
        reformatted = reformatted.replace("%r", &r);
    }

    let interfaces: Vec<InterfaceAddress> = match nix::ifaddrs::getifaddrs() {
        Ok(x) => x.collect(),
        Err(_) => vec![],
    };

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
                        out_str.push_str(&matching_network_interface(ip, &interfaces, false));
                    }
                    'd' => {
                        out_str.push_str(&matching_network_interface(ip, &interfaces, true));
                    }
                    'k' => {
                        out_str.push_str(&rbl_format(ip));
                    }
                    '%' => {
                        out_str.push('%');
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

pub fn fd_ready(fd: RawFd) -> bool {
    let s = fstat(fd);
    if let Ok(x) = s {
        if SFlag::S_IFIFO.bits() & x.st_mode == SFlag::S_IFIFO.bits() {
            return true;
        }
    }

    false
}

pub fn find_ips(reader: Box<dyn BufRead>, input_base: Option<i32>, reverse: &Reverse) -> Vec<Ip> {

    let mut ips = vec![];

    for line in reader.lines() {
        let line: String = line.as_ref().unwrap().trim().to_string();

        for part in line.split(" ") {
            let p = part.trim();
            if p == "" {
                continue;
            }

            let ip = match parse_address_mask(
                p,
                Some(32),
                Some(128),
                input_base,
                matches!(reverse, Reverse::Both | Reverse::Input),
            ) {
                Some(x) => x,
                None => {
                    eprintln!("Could not parse {}", p);
                    continue;
                }
            };
            ips.push(ip);
        }
    }
    ips
}
