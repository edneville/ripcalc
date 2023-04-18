use std::collections::HashMap;
use std::fmt;
use std::net::Ipv4Addr;
use std::net::Ipv6Addr;
use std::str::FromStr;

#[derive(Debug, PartialEq, Hash, Eq, Clone)]
pub enum Addr {
    V6(Ipv6Addr),
    V4(Ipv4Addr),
}

#[derive(Debug, PartialEq, Hash, Eq, Clone)]
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
    Hex,
    Backslash,
}

pub enum FormatProcessor {
    Percent,
    Backslash,
    None,
}

impl Ip {
    fn num_representation(&self) -> String {
        match self.address {
            Addr::V4(x) => u32::from(x).to_string(),
            Addr::V6(x) => u128::from(x).to_string(),
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

pub fn addresses<'a>(
    ip: &'a Ip,
    used: Option<&'a HashMap<Addr, bool>>,
) -> impl std::iter::Iterator<Item = Ip> + 'a {
    let b = broadcast(ip);
    let mut net = network(ip);

    std::iter::from_fn(move || {
        if let Addr::V4(mut x) = net.address {
            if let Addr::V4(y) = b.address {
                while u32::from(x) <= u32::from(y) {
                    match &used {
                        Some(map) => {
                            if map.get(&net.address).is_some() {
                                continue;
                            }
                        }
                        None => {}
                    }

                    net = Ip {
                        address: Addr::V4(Ipv4Addr::from(u32::from(x) + 1)),
                        cidr: net.cidr,
                    };

                    if let Addr::V4(a) = net.address {
                        x = a
                    };

                    return Some(Ip {
                        address: Addr::V4(Ipv4Addr::from(u32::from(x)-1)),
                        cidr: net.cidr,
                    });
                }
            }
        }

        if let Addr::V6(mut x) = net.address {
            if let Addr::V6(y) = b.address {
                while u128::from(x) < u128::from(y) {
                    match &used {
                        Some(map) => {
                            if map.get(&net.address).is_some() {
                                continue;
                            }
                        }
                        None => {}
                    }

                    net = Ip {
                        address: Addr::V6(Ipv6Addr::from(u128::from(x) + 1)),
                        cidr: net.cidr,
                    };

                    if let Addr::V6(a) = net.address {
                        x = a
                    };

                    return Some(Ip {
                        address: Addr::V6(Ipv6Addr::from(u128::from(x)-1)),
                        cidr: net.cidr,
                    });
                }
            }
        }

        None
    })
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
    let mut ip = &mut ip.clone();
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
