use std::net::Ipv4Addr;
use std::net::Ipv6Addr;
use std::str::FromStr;

#[cfg(test)]
mod test {
    use super::*;
    use ripcalc::*;

    #[test]
    fn test_broadcast_num() {
        assert_eq!(
            broadcast(&Ip {
                address: Addr::V4(Ipv4Addr::from_str("192.168.0.1").unwrap()),
                cidr: 24,
            })
            .address,
            Addr::V4(Ipv4Addr::from_str("192.168.0.255").unwrap())
        );

        assert_eq!(
            broadcast(&Ip {
                address: Addr::V4(Ipv4Addr::from_str("192.168.255.1").unwrap()),
                cidr: 24,
            })
            .address,
            Addr::V4(Ipv4Addr::from_str("192.168.255.255").unwrap())
        );

        assert_eq!(
            broadcast(&Ip {
                address: Addr::V6(
                    Ipv6Addr::from_str("2001:0db8:85a3:0000:0000:8a2e:0370:7334").unwrap()
                ),
                cidr: 64,
            })
            .address,
            Addr::V6(Ipv6Addr::from_str("2001:0db8:85a3:0000:ffff:ffff:ffff:ffff").unwrap())
        );
    }

    #[test]
    fn test_network_num() {
        assert_eq!(
            network(&Ip {
                address: Addr::V4(Ipv4Addr::from_str("192.168.0.1").unwrap()),
                cidr: 24,
            })
            .address,
            Addr::V4(Ipv4Addr::from_str("192.168.0.0").unwrap())
        );

        assert_eq!(
            network(&Ip {
                address: Addr::V4(Ipv4Addr::from_str("192.168.255.1").unwrap()),
                cidr: 24,
            })
            .address,
            Addr::V4(Ipv4Addr::from_str("192.168.255.0").unwrap())
        );

        assert_eq!(
            network(&Ip {
                address: Addr::V6(
                    Ipv6Addr::from_str("2001:0db8:85a3:0000:0000:8a2e:0370:7334").unwrap()
                ),
                cidr: 64,
            })
            .address,
            Addr::V6(Ipv6Addr::from_str("2001:0db8:85a3:0000:0000:0000:0000:0000").unwrap())
        );
    }

    #[test]
    fn test_subnet_num() {
        assert_eq!(
            subnet(&Ip {
                address: Addr::V4(Ipv4Addr::from_str("192.168.0.1").unwrap()),
                cidr: 24,
            })
            .address,
            Addr::V4(Ipv4Addr::from_str("255.255.255.0").unwrap())
        );

        assert_eq!(
            subnet(&Ip {
                address: Addr::V4(Ipv4Addr::from_str("192.168.255.1").unwrap()),
                cidr: 24,
            })
            .address,
            Addr::V4(Ipv4Addr::from_str("255.255.255.0").unwrap())
        );
    }

    #[test]
    fn test_wildcard_num() {
        assert_eq!(
            wildcard(&Ip {
                address: Addr::V4(Ipv4Addr::from_str("192.168.0.1").unwrap()),
                cidr: 24,
            })
            .address,
            Addr::V4(Ipv4Addr::from_str("0.0.0.255").unwrap())
        );

        assert_eq!(
            wildcard(&Ip {
                address: Addr::V4(Ipv4Addr::from_str("192.168.255.1").unwrap()),
                cidr: 24,
            })
            .address,
            Addr::V4(Ipv4Addr::from_str("0.0.0.255").unwrap())
        );
    }

    #[test]
    fn test_network_size() {
        assert_eq!(
            network_size(&Ip {
                address: Addr::V4(Ipv4Addr::from_str("192.168.0.0").unwrap()),
                cidr: 32,
            }),
            1
        );
        assert_eq!(
            network_size(&Ip {
                address: Addr::V4(Ipv4Addr::from_str("192.168.0.0").unwrap()),
                cidr: 31,
            }),
            2
        );
        assert_eq!(
            network_size(&Ip {
                address: Addr::V4(Ipv4Addr::from_str("192.168.0.0").unwrap()),
                cidr: 30,
            }),
            4
        );
        assert_eq!(
            network_size(&Ip {
                address: Addr::V4(Ipv4Addr::from_str("192.168.0.0").unwrap()),
                cidr: 29,
            }),
            8
        );
        assert_eq!(
            network_size(&Ip {
                address: Addr::V4(Ipv4Addr::from_str("192.168.0.0").unwrap()),
                cidr: 28,
            }),
            16
        );

        assert_eq!(
            network_size(&Ip {
                address: Addr::V6(
                    Ipv6Addr::from_str("2001:0db8:85a3:0000:0000:0000:0000:0000").unwrap()
                ),
                cidr: 128,
            }),
            1
        );

        assert_eq!(
            network_size(&Ip {
                address: Addr::V6(
                    Ipv6Addr::from_str("2001:0db8:85a3:0000:0000:0000:0000:0000").unwrap()
                ),
                cidr: 127,
            }),
            2
        );

        assert_eq!(
            network_size(&Ip {
                address: Addr::V6(
                    Ipv6Addr::from_str("2001:0db8:85a3:0000:0000:0000:0000:0000").unwrap()
                ),
                cidr: 126,
            }),
            4
        );

        assert_eq!(
            network_size(&Ip {
                address: Addr::V6(
                    Ipv6Addr::from_str("2001:0db8:85a3:0000:0000:0000:0000:0000").unwrap()
                ),
                cidr: 125,
            }),
            8
        );

        assert_eq!(
            network_size(&Ip {
                address: Addr::V6(
                    Ipv6Addr::from_str("2001:0db8:85a3:0000:0000:0000:0000:0000").unwrap()
                ),
                cidr: 124,
            }),
            16
        );

        assert_eq!(
            network_size(&Ip {
                address: Addr::V6(
                    Ipv6Addr::from_str("2001:0db8:85a3:0000:0000:0000:0000:0000").unwrap()
                ),
                cidr: 123,
            }),
            32
        );

        assert_eq!(
            network_size(&Ip {
                address: Addr::V6(
                    Ipv6Addr::from_str("2001:0db8:85a3:0000:0000:0000:0000:0000").unwrap()
                ),
                cidr: 122,
            }),
            64
        );
        assert_eq!(
            network_size(&Ip {
                address: Addr::V6(
                    Ipv6Addr::from_str("2001:0db8:85a3:0000:0000:0000:0000:0000").unwrap()
                ),
                cidr: 121,
            }),
            128
        );

        assert_eq!(
            network_size(&Ip {
                address: Addr::V6(
                    Ipv6Addr::from_str("2001:0db8:85a3:0000:0000:0000:0000:0000").unwrap()
                ),
                cidr: 120,
            }),
            256
        );
    }

    #[test]
    fn test_network_iter() {
        let net = Ip {
            address: Addr::V4(Ipv4Addr::from_str("192.168.0.0").unwrap()),
            cidr: 30,
        };

        let mut i = addresses(&net);

        assert_eq!(
            i.next().as_ref().unwrap().address,
            Addr::V4(Ipv4Addr::from_str("192.168.0.0").unwrap())
        );
        assert_eq!(
            i.next().as_ref().unwrap().address,
            Addr::V4(Ipv4Addr::from_str("192.168.0.1").unwrap())
        );
        assert_eq!(
            i.next().as_ref().unwrap().address,
            Addr::V4(Ipv4Addr::from_str("192.168.0.2").unwrap())
        );
        assert_eq!(
            i.next().as_ref().unwrap().address,
            Addr::V4(Ipv4Addr::from_str("192.168.0.3").unwrap())
        );
        assert_eq!(i.next(), None);
    }
}
