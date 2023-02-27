use std::collections::HashMap;
use std::fmt;
use std::net::Ipv4Addr;
use std::net::Ipv6Addr;

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
