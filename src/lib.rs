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

impl Ip {
    fn num_representation(&self) -> String {
        match self.address {
            Addr::V4(x) => u32::from(x).to_string(),
            Addr::V6(x) => u128::from(x).to_string(),
        }
    }
    fn hex_quad_representation(&self) -> String {
        match self.address {
            Addr::V4(x) => {
                let mut s: String = "".to_string();
                let octet = x.octets();
                for (i,o) in octet.iter().enumerate() {
                    s.push_str(&format!("{:02x}", o));
                    if i % 2 == 1 && i != 3 {
                        s.push_str(&":");
                    }
                }
                s.to_string()
            },
            Addr::V6(x) => {
                let mut s: String = "".to_string();
                let octet = x.octets();
                for (i,o) in octet.iter().enumerate() {
                    s.push_str(&format!("{:02x}", o));
                    if i % 2 == 1 && i != 15 {
                        s.push_str(&":");
                    }
                }
                s.to_string()
            },
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
            bin = (!(bin) & u32::from(x)) as u32;
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
            bin = (!(bin) & u128::from(x)) as u128;
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

    rows.insert( Ip { address: Addr::V4(Ipv4Addr::from_str( "0.0.0.0").unwrap()), cidr: 8}, "Current network".to_string() );
    rows.insert( Ip { address: Addr::V4(Ipv4Addr::from_str( "10.0.0.0").unwrap()), cidr: 8}, "Used for local communications within a private network.".to_string() );
    rows.insert( Ip { address: Addr::V4(Ipv4Addr::from_str( "100.64.0.0").unwrap()), cidr: 10}, "Shared address space for communications between a service provider and its subscribers when using a carrier-grade NAT.".to_string() ); 
    rows.insert( Ip { address: Addr::V4(Ipv4Addr::from_str( "127.0.0.0").unwrap()), cidr: 8}, "Used for loopback addresses to the local host.".to_string() );
    rows.insert( Ip { address: Addr::V4(Ipv4Addr::from_str( "169.254.0.0").unwrap()), cidr: 16}, "Used for link-local addresses between two hosts on a single link when no IP address is otherwise specified, such as would have normally been retrieved from a DHCP server.".to_string() );
    rows.insert( Ip { address: Addr::V4(Ipv4Addr::from_str( "172.16.0.0").unwrap()), cidr: 12}, "Used for local communications within a private network.".to_string() );
    rows.insert( Ip { address: Addr::V4(Ipv4Addr::from_str( "192.0.0.0").unwrap()), cidr: 24}, "IETF Protocol Assignments.".to_string() );
    rows.insert( Ip { address: Addr::V4(Ipv4Addr::from_str( "192.0.2.0").unwrap()), cidr: 24}, "Assigned as TEST-NET-1, documentation and examples.".to_string() );
    rows.insert( Ip { address: Addr::V4(Ipv4Addr::from_str( "192.88.99.0").unwrap()), cidr: 24}, "Reserved. Formerly used for IPv6 to IPv4 relay (included IPv6 address block 2002::/16).".to_string() ); 
    rows.insert( Ip { address: Addr::V4(Ipv4Addr::from_str( "192.168.0.0").unwrap()), cidr: 16}, "Used for local communications within a private network.".to_string() );
    rows.insert( Ip { address: Addr::V4(Ipv4Addr::from_str( "198.18.0.0").unwrap()), cidr: 15}, "Used for benchmark testing of inter-network communications between two separate subnets.".to_string() );
    rows.insert( Ip { address: Addr::V4(Ipv4Addr::from_str( "198.51.100.0").unwrap()), cidr: 24}, "Assigned as TEST-NET-2, documentation and examples.".to_string() );
    rows.insert( Ip { address: Addr::V4(Ipv4Addr::from_str( "203.0.113.0").unwrap()), cidr: 24}, "Assigned as TEST-NET-3, documentation and examples.".to_string() );
    rows.insert( Ip { address: Addr::V4(Ipv4Addr::from_str( "224.0.0.0").unwrap()), cidr: 4}, "In use for IP multicast. (Former Class D network.)".to_string() );
    rows.insert( Ip { address: Addr::V4(Ipv4Addr::from_str( "233.252.0.0").unwrap()), cidr: 24}, "Assigned as MCAST-TEST-NET, documentation and examples.".to_string() );
    rows.insert( Ip { address: Addr::V4(Ipv4Addr::from_str( "240.0.0.0").unwrap()), cidr: 4}, "Reserved for future use. (Former Class E network.)".to_string() ); 
    rows.insert( Ip { address: Addr::V4(Ipv4Addr::from_str( "255.255.255.255").unwrap()), cidr: 32}, "Reserved for limited broadcast destination address.".to_string() );


    rows.insert( Ip { address: Addr::V6(Ipv6Addr::from_str( "::1").unwrap()), cidr: 128}, "Loopback address".to_string() );
    rows.insert( Ip { address: Addr::V6(Ipv6Addr::from_str( "::ffff:0:0").unwrap()), cidr: 96}, "IPv4-mapped addresses".to_string() );
    rows.insert( Ip { address: Addr::V6(Ipv6Addr::from_str( "::ffff:0:0:0").unwrap()), cidr: 96}, "IPv4 translated addresses".to_string() );
    rows.insert( Ip { address: Addr::V6(Ipv6Addr::from_str( "64:ff9b::").unwrap()), cidr: 96}, "IPv4/IPv6 translation".to_string() );
    rows.insert( Ip { address: Addr::V6(Ipv6Addr::from_str( "64:ff9b:1::").unwrap()), cidr: 48}, "IPv4/IPv6 translation".to_string() );
    rows.insert( Ip { address: Addr::V6(Ipv6Addr::from_str( "100::").unwrap()), cidr: 64}, "Discard prefix".to_string() );
    rows.insert( Ip { address: Addr::V6(Ipv6Addr::from_str( "2001:0000::").unwrap()), cidr: 32}, "Teredo tunneling".to_string() );
    rows.insert( Ip { address: Addr::V6(Ipv6Addr::from_str( "2001:20::").unwrap()), cidr: 28}, "ORCHIDv2".to_string() );
    rows.insert( Ip { address: Addr::V6(Ipv6Addr::from_str( "2001:db8::").unwrap()), cidr: 32}, "Documentation range".to_string() );
    rows.insert( Ip { address: Addr::V6(Ipv6Addr::from_str( "2002::").unwrap()), cidr: 16}, "The 6to4 addressing scheme (legacy)".to_string() );
    rows.insert( Ip { address: Addr::V6(Ipv6Addr::from_str( "fc00::").unwrap()), cidr: 7}, "Unique local address".to_string() );
    rows.insert( Ip { address: Addr::V6(Ipv6Addr::from_str( "ff00::").unwrap()), cidr: 8}, "Multicast address".to_string() );
    rows.insert( Ip { address: Addr::V6(Ipv6Addr::from_str( "fe80::").unwrap()), cidr: 10}, "Link local".to_string() );

    match ip.address {
        Addr::V4(x) => {
            let search_ip = x;

            for i in 0..32 {
                let cidr = 32 - i;

                let net_row = rows.get(&network(&Ip { address: Addr::V4(search_ip), cidr, }));

                match net_row {
                    Some(s) => return Some(s.to_string()),
                    None => {},
                }
            }
        },
        Addr::V6(x) => {
            let search_ip = x;

            for i in 0..128 {
                let cidr = 128 - i;

                let net_row = rows.get(&network(&Ip { address: Addr::V6(search_ip), cidr, }));

                match net_row {
                    Some(s) => return Some(s.to_string()),
                    None => {},
                }
            }
        },
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

pub fn format_details(ip: &Ip, formatted: String, rows: &Option<HashMap<Ip, NetRow>>) -> Option<String> {
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

    match network_reservation(ip) {
        Some(r) => reformatted = reformatted.replace("%r", &r),
        None => {},
    }

    Some(reformatted
        .replace("\\n", "\n")
        .replace("%a", &ip.to_string())
        .replace("%xa", &ip.hex_quad_representation())
        .replace("%c", &ip.cidr.to_string())
        .replace("%la", &ip.num_representation())
        .replace("%b", &b.to_string())
        .replace("%xb", &b.hex_quad_representation())
        .replace("%lb", &b.num_representation())
        .replace("%n", &n.to_string())
        .replace("%xn", &n.hex_quad_representation())
        .replace("%ln", &n.num_representation())
        .replace("%s", &s.to_string())
        .replace("%xs", &s.hex_quad_representation())
        .replace("%ls", &s.num_representation())
        .replace("%w", &w.to_string())
        .replace("%xw", &w.hex_quad_representation())
        .replace("%lw", &w.num_representation())
        .replace("%t", &network_size(ip).to_string())
    )
}
