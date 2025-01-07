use std::collections::HashMap;
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

        let mut i = addresses(&net, None, None);

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

    #[test]
    fn test_format_ng_ip() {
        let net = Ip {
            address: Addr::V4(Ipv4Addr::from_str("192.168.0.0").unwrap()),
            cidr: 30,
        };

        let f = format_details(&net, "%a".to_string(), &None, None, None);

        assert_eq!(f, Some("192.168.0.0".to_string()));
    }

    #[test]
    fn test_format_ng_percent() {
        let f = format_details(
            &Ip {
                address: Addr::V4(Ipv4Addr::from_str("192.168.0.0").unwrap()),
                cidr: 30,
            },
            "%".to_string(),
            &None,
            None,
            None,
        );

        assert_eq!(f, Some("%".to_string()));

        let f = format_details(
            &Ip {
                address: Addr::V4(Ipv4Addr::from_str("192.168.0.0").unwrap()),
                cidr: 30,
            },
            "%%".to_string(),
            &None,
            None,
            None,
        );

        assert_eq!(f, Some("%".to_string()));

        let f = format_details(
            &Ip {
                address: Addr::V4(Ipv4Addr::from_str("192.168.0.0").unwrap()),
                cidr: 30,
            },
            "%%%".to_string(),
            &None,
            None,
            None,
        );

        assert_eq!(f, Some("%%".to_string()));

        let f = format_details(
            &Ip {
                address: Addr::V4(Ipv4Addr::from_str("192.168.0.0").unwrap()),
                cidr: 30,
            },
            "%%%%".to_string(),
            &None,
            None,
            None,
        );

        assert_eq!(f, Some("%%".to_string()));
    }

    #[test]
    fn test_format_ng_boilerplate() {
        let net = Ip {
            address: Addr::V6(Ipv6Addr::from_str("2001:ba8:1f1:f1cb::4").unwrap()),
            cidr: 64,
        };

        let f = format_details(&net, "select * from IP6 where (ip >= %ln and ip <= %lb) and active = 1;\nupdate IP6 set active = 0 where (ip >= %ln and ip <= %lb) and active = 1;".to_string(), &None, None, None);

        assert_eq!(f, Some("select * from IP6 where (ip >= 42540724579414763292693624807812497408 and ip <= 42540724579414763311140368881522049023) and active = 1;
update IP6 set active = 0 where (ip >= 42540724579414763292693624807812497408 and ip <= 42540724579414763311140368881522049023) and active = 1;".to_string()));
    }

    #[test]
    fn test_format_ng_percent_marker() {
        let net = Ip {
            address: Addr::V6(Ipv6Addr::from_str("2001:ba8:1f1:f1cb::4").unwrap()),
            cidr: 64,
        };

        let f = format_details(&net, "%%b".to_string(), &None, None, None);

        assert_eq!(f, Some("%b".to_string()));
    }

    #[test]
    fn test_format_ng_long_broadcast() {
        let net = Ip {
            address: Addr::V6(Ipv6Addr::from_str("2001:ba8:1f1:f1cb::4").unwrap()),
            cidr: 64,
        };

        let f = format_details(&net, "%lb".to_string(), &None, None, None);

        assert_eq!(
            f,
            Some("42540724579414763311140368881522049023".to_string())
        );
    }

    #[test]
    fn test_format_ng_long_broadcast_backslash() {
        let net = Ip {
            address: Addr::V6(Ipv6Addr::from_str("2001:ba8:1f1:f1cb::4").unwrap()),
            cidr: 64,
        };

        let f = format_details(&net, "%lb\n\n\n%%".to_string(), &None, None, None);

        assert_eq!(
            f,
            Some("42540724579414763311140368881522049023\n\n\n%".to_string())
        );
    }

    #[test]
    fn test_format_ng_backslash() {
        let net = Ip {
            address: Addr::V6(Ipv6Addr::from_str("2001:ba8:1f1:f1cb::4").unwrap()),
            cidr: 64,
        };

        let f = format_details(&net, "\n".to_string(), &None, None, None);
        assert_eq!(f, Some("\n".to_string()));

        let f = format_details(&net, "\\".to_string(), &None, None, None);
        assert_eq!(f, Some('\\'.to_string()));

        let f = format_details(&net, "\\i".to_string(), &None, None, None);
        assert_eq!(f, Some("i".to_string()));

        let f = format_details(&net, "\\t".to_string(), &None, None, None);
        assert_eq!(f, Some("\t".to_string()));
    }

    #[test]
    fn test_address_space_use() {
        let net = Ip {
            address: Addr::V4(Ipv4Addr::from_str("192.168.0.0").unwrap()),
            cidr: 30,
        };

        let mut hm: HashMap<Addr, bool> = HashMap::new();

        hm.insert(Addr::V4(Ipv4Addr::from_str("192.168.0.0").unwrap()), true);
        hm.insert(Addr::V4(Ipv4Addr::from_str("192.168.0.1").unwrap()), true);
        hm.insert(Addr::V4(Ipv4Addr::from_str("192.168.0.2").unwrap()), true);

        let mut i = addresses(&net, Some(&hm), None);

        assert_eq!(
            i.next().as_ref().unwrap().address,
            Addr::V4(Ipv4Addr::from_str("192.168.0.3").unwrap())
        );
        assert_eq!(i.next(), None);

        let mut hm: HashMap<Addr, bool> = HashMap::new();

        hm.insert(Addr::V4(Ipv4Addr::from_str("192.168.1.0").unwrap()), true);
        hm.insert(Addr::V4(Ipv4Addr::from_str("192.168.1.1").unwrap()), true);
        hm.insert(Addr::V4(Ipv4Addr::from_str("192.168.1.2").unwrap()), true);

        let mut i = addresses(&net, Some(&hm), None);

        assert_eq!(
            i.next().as_ref().unwrap().address,
            Addr::V4(Ipv4Addr::from_str("192.168.0.0").unwrap())
        );
    }

    #[test]
    fn test_smallest_network() {
        let mut hm: HashMap<Ip, bool> = HashMap::new();

        let net = Ip {
            address: Addr::V4(Ipv4Addr::from_str("192.168.0.0").unwrap()),
            cidr: 30,
        };
        hm.insert(net, true);

        let net = Ip {
            address: Addr::V4(Ipv4Addr::from_str("192.168.1.0").unwrap()),
            cidr: 24,
        };
        hm.insert(net, true);

        let net = Ip {
            address: Addr::V4(Ipv4Addr::from_str("192.168.2.0").unwrap()),
            cidr: 24,
        };
        hm.insert(net, true);
        let net = Ip {
            address: Addr::V4(Ipv4Addr::from_str("192.168.3.0").unwrap()),
            cidr: 24,
        };
        hm.insert(net, true);

        let net = Ip {
            address: Addr::V4(Ipv4Addr::from_str("192.168.4.0").unwrap()),
            cidr: 24,
        };
        hm.insert(net, true);

        let net = Ip {
            address: Addr::V4(Ipv4Addr::from_str("192.168.5.0").unwrap()),
            cidr: 24,
        };
        hm.insert(net, true);

        let net = Ip {
            address: Addr::V4(Ipv4Addr::from_str("192.168.2.0").unwrap()),
            cidr: 24,
        };
        hm.insert(net, true);

        assert_eq!(
            smallest_group_network(&hm),
            Some(Ip {
                address: Addr::V4(Ipv4Addr::from_str("192.168.0.0").unwrap()),
                cidr: 21,
            })
        );

        let mut hm: HashMap<Ip, bool> = HashMap::new();

        let net = Ip {
            address: Addr::V4(Ipv4Addr::from_str("192.168.0.0").unwrap()),
            cidr: 24,
        };
        hm.insert(net, true);
        let net = Ip {
            address: Addr::V4(Ipv4Addr::from_str("192.168.1.0").unwrap()),
            cidr: 24,
        };
        hm.insert(net, true);
        let net = Ip {
            address: Addr::V4(Ipv4Addr::from_str("192.168.20.0").unwrap()),
            cidr: 24,
        };
        hm.insert(net, true);
        let net = Ip {
            address: Addr::V4(Ipv4Addr::from_str("192.168.40.0").unwrap()),
            cidr: 24,
        };
        hm.insert(net, true);
        let net = Ip {
            address: Addr::V4(Ipv4Addr::from_str("192.168.70.0").unwrap()),
            cidr: 24,
        };
        hm.insert(net, true);
        let net = Ip {
            address: Addr::V4(Ipv4Addr::from_str("192.168.90.0").unwrap()),
            cidr: 24,
        };
        hm.insert(net, true);
        let net = Ip {
            address: Addr::V4(Ipv4Addr::from_str("192.168.200.0").unwrap()),
            cidr: 24,
        };
        hm.insert(net, true);
        let net = Ip {
            address: Addr::V4(Ipv4Addr::from_str("192.168.255.0").unwrap()),
            cidr: 24,
        };
        hm.insert(net, true);

        let net = Ip {
            address: Addr::V4(Ipv4Addr::from_str("192.168.220.0").unwrap()),
            cidr: 24,
        };
        hm.insert(net, true);

        assert_eq!(
            smallest_group_network(&hm),
            Some(Ip {
                address: Addr::V4(Ipv4Addr::from_str("192.168.0.0").unwrap()),
                cidr: 16,
            })
        );

        let mut hm: HashMap<Ip, bool> = HashMap::new();
        let net = Ip {
            address: Addr::V4(Ipv4Addr::from_str("192.168.0.0").unwrap()),
            cidr: 30,
        };
        hm.insert(net, true);
        let net = Ip {
            address: Addr::V4(Ipv4Addr::from_str("192.168.0.1").unwrap()),
            cidr: 30,
        };
        hm.insert(net, true);
        let net = Ip {
            address: Addr::V4(Ipv4Addr::from_str("192.168.0.2").unwrap()),
            cidr: 30,
        };
        hm.insert(net, true);

        assert_eq!(
            smallest_group_network(&hm),
            Some(Ip {
                address: Addr::V4(Ipv4Addr::from_str("192.168.0.0").unwrap()),
                cidr: 30,
            })
        );
    }

    #[test]
    fn test_base_hex() {
        assert_eq!(
            parse_address_mask("192.168.1.1", None, None, Some(10), false),
            Some(Ip {
                address: Addr::V4(Ipv4Addr::from_str("192.168.1.1").unwrap()),
                cidr: 24,
            })
        );

        assert_eq!(
            parse_address_mask("192.168.1.1", None, None, None, false),
            Some(Ip {
                address: Addr::V4(Ipv4Addr::from_str("192.168.1.1").unwrap()),
                cidr: 24,
            })
        );
        assert_eq!(
            parse_address_mask("D4166001", None, None, Some(16), false),
            Some(Ip {
                address: Addr::V4(Ipv4Addr::from_str("212.22.96.1").unwrap()),
                cidr: 24,
            })
        );
        assert_eq!(
            parse_address_mask("177.0.0.1", None, None, Some(8), false),
            Some(Ip {
                address: Addr::V4(Ipv4Addr::from_str("127.0.0.1").unwrap()),
                cidr: 24,
            })
        );
    }

    #[test]
    fn test_reverse() {
        assert_eq!(
            parse_address_mask("0101A8C0", None, None, Some(16), true),
            Some(Ip {
                address: Addr::V4(Ipv4Addr::from_str("192.168.1.1").unwrap()),
                cidr: 24,
            })
        );
    }

    #[test]
    fn test_signed_ints() {
        let net = Ip {
            address: Addr::V4(Ipv4Addr::from_str("147.147.137.206").unwrap()),
            cidr: 30,
        };

        let f = format_details(&net, "%La".to_string(), &None, None, None);
        assert_eq!(f, Some("-1819047474".to_string()));
        let f = format_details(&net, "%la".to_string(), &None, None, None);
        assert_eq!(f, Some("2475919822".to_string()));

        let net = Ip {
            address: Addr::V6(Ipv6Addr::from_str("ffff::ffff").unwrap()),
            cidr: 30,
        };

        let f = format_details(&net, "%La".to_string(), &None, None, None);
        assert_eq!(f, Some("-5192296858534827628530496329154561".to_string()));
        let f = format_details(&net, "%la".to_string(), &None, None, None);
        assert_eq!(
            f,
            Some("340277174624079928635746076935439056895".to_string())
        );
    }

    #[test]
    fn test_within_ipv4() {
        assert_eq!(
            within(
                &Ip {
                    address: Addr::V4(Ipv4Addr::from_str("127.0.0.1").unwrap()),
                    cidr: 8
                },
                &Ip {
                    address: Addr::V4(Ipv4Addr::from_str("10.0.0.0").unwrap()),
                    cidr: 8
                },
            ),
            false
        );
        assert_eq!(
            within(
                &Ip {
                    address: Addr::V4(Ipv4Addr::from_str("192.168.0.0").unwrap()),
                    cidr: 24
                },
                &Ip {
                    address: Addr::V4(Ipv4Addr::from_str("192.168.0.0").unwrap()),
                    cidr: 16
                },
            ),
            false
        );
        assert_eq!(
            within(
                &Ip {
                    address: Addr::V4(Ipv4Addr::from_str("127.0.0.1").unwrap()),
                    cidr: 8
                },
                &Ip {
                    address: Addr::V4(Ipv4Addr::from_str("192.168.0.0").unwrap()),
                    cidr: 16
                },
            ),
            false
        );
        assert_eq!(
            within(
                &Ip {
                    address: Addr::V4(Ipv4Addr::from_str("127.0.0.1").unwrap()),
                    cidr: 8
                },
                &Ip {
                    address: Addr::V4(Ipv4Addr::from_str("127.1.1.1").unwrap()),
                    cidr: 16
                },
            ),
            true
        );
    }
    #[test]
    fn test_within_ipv6() {
        assert_eq!(
            within(
                &Ip {
                    address: Addr::V6(Ipv6Addr::from_str("::1").unwrap()),
                    cidr: 48
                },
                &Ip {
                    address: Addr::V6(Ipv6Addr::from_str("f::1").unwrap()),
                    cidr: 48
                },
            ),
            false
        );
        assert_eq!(
            within(
                &Ip {
                    address: Addr::V6(Ipv6Addr::from_str("::1").unwrap()),
                    cidr: 48
                },
                &Ip {
                    address: Addr::V6(Ipv6Addr::from_str("dead:beef::cafe").unwrap()),
                    cidr: 48
                },
            ),
            false
        );
        assert_eq!(
            within(
                &Ip {
                    address: Addr::V6(Ipv6Addr::from_str("1::1").unwrap()),
                    cidr: 48
                },
                &Ip {
                    address: Addr::V6(Ipv6Addr::from_str("::f:f:f:f:f").unwrap()),
                    cidr: 48
                },
            ),
            false
        );
    }

    #[test]
    fn test_network_divide_iter() {
        let net = Ip {
            address: Addr::V4(Ipv4Addr::from_str("192.168.0.0").unwrap()),
            cidr: 24,
        };

        let mut i = addresses(&net, None, Some(25));
        assert_eq!(
            i.next().as_ref().unwrap().address,
            Addr::V4(Ipv4Addr::from_str("192.168.0.0").unwrap())
        );
        assert_eq!(
            i.next().as_ref().unwrap().address,
            Addr::V4(Ipv4Addr::from_str("192.168.0.128").unwrap())
        );
        assert_eq!(i.next(), None,);

        let mut i = addresses(&net, None, Some(26));
        assert_eq!(
            i.next().as_ref().unwrap().address,
            Addr::V4(Ipv4Addr::from_str("192.168.0.0").unwrap())
        );
        assert_eq!(
            i.next().as_ref().unwrap().address,
            Addr::V4(Ipv4Addr::from_str("192.168.0.64").unwrap())
        );
        assert_eq!(
            i.next().as_ref().unwrap().address,
            Addr::V4(Ipv4Addr::from_str("192.168.0.128").unwrap())
        );
        assert_eq!(
            i.next().as_ref().unwrap().address,
            Addr::V4(Ipv4Addr::from_str("192.168.0.192").unwrap())
        );
        assert_eq!(i.next(), None,);
    }

    #[test]
    fn test_network_divide_iter_v6() {
        let net = Ip {
            address: Addr::V6(Ipv6Addr::from_str("::1").unwrap()),
            cidr: 48,
        };

        let mut i = addresses(&net, None, Some(64));
        assert_eq!(
            i.next().as_ref().unwrap().address,
            Addr::V6(Ipv6Addr::from_str("::").unwrap())
        );
        assert_eq!(
            i.next().as_ref().unwrap().address,
            Addr::V6(Ipv6Addr::from_str("0:0:0:1::").unwrap())
        );
        assert_eq!(
            i.next().as_ref().unwrap().address,
            Addr::V6(Ipv6Addr::from_str("0:0:0:2::").unwrap())
        );

        let i = addresses(&net, None, Some(64));
        let v: Vec<Ip> = i.collect();
        assert_eq!(v.len(), 65536);
    }

    #[test]
    fn test_networks_sizing_v6() {
        let net = Ip {
            address: Addr::V6(Ipv6Addr::from_str("::1").unwrap()),
            cidr: 48,
        };

        assert_eq!(subnets_in_network(64, &net), 65536);
        assert_eq!(subnets_in_network(48, &net), 1);
        assert_eq!(subnets_in_network(49, &net), 2);
    }

    #[test]
    fn test_networks_sizing_v4() {
        let net = Ip {
            address: Addr::V4(Ipv4Addr::from_str("127.0.0.1").unwrap()),
            cidr: 16,
        };

        assert_eq!(subnets_in_network(24, &net), 256);
        assert_eq!(subnets_in_network(25, &net), 512);
        assert_eq!(subnets_in_network(26, &net), 1024);
        assert_eq!(subnets_in_network(27, &net), 2048);
        assert_eq!(subnets_in_network(28, &net), 4096);
    }

    #[test]
    fn test_smallest_network_limited() {
        let empty: HashMap<Ip, bool> = HashMap::new();
        assert_eq!(smallest_group_network_limited(&empty, 32), None);
    }

    #[test]
    fn test_smallest_network_limited_22_24() {
        let mut net_list: HashMap<Ip, bool> = HashMap::new();
        for i in 0..4 {
            for j in 0..4 {
                net_list.insert(
                    Ip {
                        address: Addr::V4(
                            Ipv4Addr::from_str(&format!("192.168.{i}.{j}")).expect("bad ipv4"),
                        ),
                        cidr: 24,
                    },
                    true,
                );
            }
        }

        let mut resp = smallest_group_network_limited(&net_list, 22).unwrap();
        resp.sort_by(|a, b| a.partial_cmp(b).unwrap());
        assert_eq!(
            resp,
            [Ip {
                address: Addr::V4(Ipv4Addr::from_str(&format!("192.168.0.0")).expect("bad ipv4")),
                cidr: 22
            }]
        );
    }

    #[test]
    fn test_smallest_network_limited_22_8() {
        let mut net_list: HashMap<Ip, bool> = HashMap::new();
        for i in 0..4 {
            for j in 0..4 {
                net_list.insert(
                    Ip {
                        address: Addr::V4(
                            Ipv4Addr::from_str(&format!("192.{i}.{j}.0")).expect("bad ipv4"),
                        ),
                        cidr: 8,
                    },
                    true,
                );
            }
        }

        let mut resp = smallest_group_network_limited(&net_list, 22).unwrap();
        resp.sort_by(|a, b| a.partial_cmp(b).unwrap());
        assert_eq!(
            resp,
            [
                Ip {
                    address: Addr::V4(Ipv4Addr::from_str(&format!("192.0.0.0")).expect("bad ipv4")),
                    cidr: 22
                },
                Ip {
                    address: Addr::V4(Ipv4Addr::from_str(&format!("192.1.0.0")).expect("bad ipv4")),
                    cidr: 22
                },
                Ip {
                    address: Addr::V4(Ipv4Addr::from_str(&format!("192.2.0.0")).expect("bad ipv4")),
                    cidr: 22
                },
                Ip {
                    address: Addr::V4(Ipv4Addr::from_str(&format!("192.3.0.0")).expect("bad ipv4")),
                    cidr: 22
                },
            ]
        );
    }

    #[test]
    fn test_smallest_network_limited_24_8() {
        let mut net_list: HashMap<Ip, bool> = HashMap::new();
        for i in 0..4 {
            for j in 0..4 {
                net_list.insert(
                    Ip {
                        address: Addr::V4(
                            Ipv4Addr::from_str(&format!("192.0.{i}.{j}")).expect("bad ipv4"),
                        ),
                        cidr: 8,
                    },
                    true,
                );
            }
        }

        let mut resp = smallest_group_network_limited(&net_list, 24).unwrap();
        resp.sort_by(|a, b| a.partial_cmp(b).unwrap());
        assert_eq!(
            resp,
            [
                Ip {
                    address: Addr::V4(Ipv4Addr::from_str(&format!("192.0.0.0")).expect("bad ipv4")),
                    cidr: 24
                },
                Ip {
                    address: Addr::V4(Ipv4Addr::from_str(&format!("192.0.1.0")).expect("bad ipv4")),
                    cidr: 24
                },
                Ip {
                    address: Addr::V4(Ipv4Addr::from_str(&format!("192.0.2.0")).expect("bad ipv4")),
                    cidr: 24
                },
                Ip {
                    address: Addr::V4(Ipv4Addr::from_str(&format!("192.0.3.0")).expect("bad ipv4")),
                    cidr: 24
                },
            ]
        );
    }
}
