use getopts::Options;
use ripcalc::*;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::net::Ipv4Addr;
use std::net::Ipv6Addr;
use std::net::ToSocketAddrs;
use std::str::FromStr;

fn print_details(
    ip: &Ip,
    matches: &getopts::Matches,
    rows: &Option<HashMap<Ip, NetRow>>,
    used: Option<&HashMap<Addr, bool>>,
) {
    let mut formatted = if matches.opt_present("f") {
        matches.opt_str("f").unwrap()
    } else {
        match ip.address {
            Addr::V4(_) => "IP is: %a/%c\nBroadcast is: %b\nNetwork is: %n\nSubnet is: %s\nWildcard is: %w\nNetwork size: %t\n".to_string(),
            Addr::V6(_) => "IP is: %a/%c\nExpanded: %xa\nNetwork is: %xn\nLast host address: %xb\nSubnet is: %s\nNetwork size: %t\n".to_string(),
        }
    };

    if formatted == "cidr" {
        formatted = "%a/%c\n".to_string();
    }

    if formatted == "short" {
        formatted = "%a\n".to_string();
    }

    if matches.opt_present("l") {
        for ip_copy in addresses(ip, used) {
            if let Some(m) = format_details(&ip_copy, formatted.to_string(), rows) {
                print!("{}", m);
            }
        }
        return;
    }

    if let Some(m) = format_details(ip, formatted, rows) {
        print!("{}", m);
    }
}

fn process_csv(
    mut reader: csv::Reader<File>,
    field_name: String,
    rows: &mut Option<HashMap<Ip, NetRow>>,
) {
    let headers = reader.headers().unwrap();
    let mut header_names: Vec<String> = vec![];
    let mut field_num: Option<usize> = None;
    for i in 0..headers.len() {
        if headers[i] == field_name {
            field_num = Some(i);
        }
        header_names.push(headers[i].to_string());
    }
    if field_num.is_none() {
        eprintln!("Cannot find csv field {}", field_name);
        std::process::exit(1);
    }

    let field_num = field_num.unwrap();

    for result in reader.records() {
        let record = result.unwrap();

        if record.get(field_num).is_some() {
            let rec = record.get(field_num).unwrap();
            let parts: Vec<&str> = rec.split('/').collect();
            if parts.len() < 2 {
                eprintln!("{}: not in ip/cidr format", rec);
                continue;
            }

            let mut row_ip: Option<Addr> = None;

            if parts[0].contains(':') {
                if rows.is_none() {
                    *rows = Some(HashMap::new());
                }

                let v6 = Ipv6Addr::from_str(parts[0]);
                if v6.is_err() {
                    eprintln!("{}: not in ip/cidr format", rec);
                    continue;
                }
                row_ip = Some(Addr::V6(v6.unwrap()));
            }

            if parts[0].contains('.') {
                if rows.is_none() {
                    *rows = Some(HashMap::new());
                }

                let v4 = Ipv4Addr::from_str(parts[0]);
                if v4.is_err() {
                    eprintln!("{}: not in ip/cidr format", rec);
                    continue;
                }
                row_ip = Some(Addr::V4(v4.unwrap()));
            }

            let cidr = parts[1].to_string().parse::<u32>();
            if cidr.is_err() {
                eprintln!("{}: not in ip/cidr format", rec);
                continue;
            }
            let cidr = cidr.unwrap();

            if row_ip.is_none() {
                continue;
            }

            let ip = Ip {
                address: row_ip.unwrap(),
                cidr,
            };

            let mut hm = HashMap::new();
            for i in 0..header_names.len() {
                hm.insert(header_names[i].to_string(), record[i].to_string());
            }

            rows.as_mut()
                .unwrap()
                .insert(ip.clone(), NetRow { row: hm.clone() });
        }
    }
}

fn parse_mask(mask: &str) -> Option<u32> {
    match mask.parse::<u32>() {
        Ok(n) => Some(n),
        Err(_) => None,
    }
}

fn parse_v6(address: &str) -> Option<Addr> {
    match Ipv6Addr::from_str(address) {
        Ok(i) => Some(Addr::V6(i)),
        Err(_) => None,
    }
}

fn parse_v4(address: &str) -> Option<Addr> {
    match Ipv4Addr::from_str(address) {
        Ok(i) => Some(Addr::V4(i)),
        Err(_) => None,
    }
}

fn parse_v4_v6(address: &str) -> Option<Addr> {
    if address.find(':').is_some() {
        return parse_v6(address);
    }

    if address.find('.').is_some() {
        return parse_v4(address);
    }

    None
}

fn parse_address_mask(a: &str) -> Option<Ip> {
    let parts: Vec<&str> = a.split('/').collect();

    let mut arg = parts[0];

    let mut input_mask = 24;
    if parts.len() > 1 {
        if let Some(m) = parse_mask(parts[1]) {
            input_mask = m;
        }
    };

    let input_ip = parse_v4_v6(arg);

    if let Some(input_ip) = input_ip {
        return Some(Ip {
            address: input_ip,
            cidr: input_mask,
        });
    }

    let addrs_iter = format!("{}:443", arg).to_socket_addrs();
    let mut buffer: String;

    if let Ok(mut address) = addrs_iter {
        buffer = format!("{}", address.next().unwrap());
        let v: Vec<&str> = buffer.split(':').collect();
        buffer = v[0].to_string();
        arg = buffer.as_str();
    }

    let input_ip = parse_v4_v6(arg);

    if let Some(input_ip) = input_ip {
        return Some(Ip {
            address: input_ip,
            cidr: input_mask,
        });
    }

    None
}

fn main() {
    let mut opts = Options::new();
    let mut rows: Option<HashMap<Ip, NetRow>> = None;
    let mut input_ip: Option<Addr> = None;
    let mut input_mask: Option<u32> = None;
    let args: Vec<String> = std::env::args().collect();
    opts.parsing_style(getopts::ParsingStyle::FloatingFrees);
    opts.optopt("4", "ipv4", "ipv4 address", "IPv4");
    opts.optopt("6", "ipv6", "ipv6 address", "IPv6");
    opts.optopt("f", "format", "format output\n'cidr' expands to %a/%c\\n\n'short' expands to %a\\n\nSee manual for more options", "STRING");
    opts.optopt("m", "mask", "cidr mask", "CIDR");
    opts.optopt("c", "csv", "csv reference file", "PATH");
    opts.optopt("i", "field", "csv field", "FIELD");
    opts.optflag("l", "list", "list all addresses in network");
    opts.optflag("h", "help", "display help");
    opts.optflag("a", "available", "display unused addresses");
    opts.optopt("s", "file", "lookup addresses from, - for stdin", "PATH");

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(f) => {
            println!("{}", f);
            std::process::exit(1);
        }
    };

    if matches.opt_present("h") {
        println!("{}", opts.usage("ripcalc"));
        std::process::exit(0);
    }

    if matches.opt_present("c") {
        let path = matches.opt_str("c").unwrap();
        let reader = match csv::Reader::from_path(&path) {
            Ok(r) => r,
            Err(x) => {
                eprintln!("Cannot open {}: {}", &path, x);
                std::process::exit(1);
            },
        };
        let field_name = if matches.opt_present("i") {
            matches.opt_str("i").unwrap()
        } else {
            "network".to_string()
        };

        process_csv(reader, field_name, &mut rows);
    }

    if let Some(v) = matches.opt_str("4") {
        input_ip = parse_v4(&v);
    }

    if let Some(v) = matches.opt_str("6") {
        input_ip = parse_v6(&v);
    }

    let free_arg = matches.free.clone();
    if !free_arg.is_empty() {
        let arg = free_arg[0].to_string();

        let ip = parse_address_mask(&arg);

        if let Some(ip) = ip {
            input_ip = Some(ip.address);
            input_mask = Some(ip.cidr);
        }
    }

    if let Some(v) = matches.opt_str("m") {
        input_mask = parse_mask(&v);
    }

    if matches.opt_str("s").is_some() {
        let path = matches.opt_str("s").unwrap();
        let reader: Box<dyn BufRead> = if path == "-" {
            Box::new(BufReader::new(std::io::stdin()))
        } else {
            Box::new(BufReader::new(File::open(path).unwrap()))
        };

        let mut used: HashMap<Addr, bool> = HashMap::new();
        if matches.opt_present("a") {
            for line in reader.lines() {
                let ip = match parse_address_mask(&line.as_ref().unwrap()) {
                    Some(x) => { x },
                    None => {
                        eprintln!("Could not parse {}", &line.as_ref().unwrap());
                        continue;
                    }
                };
                used.insert(ip.address.clone(), true);
            }
            if input_ip.is_none() || input_mask.is_none() {
                eprintln!("No network specified");
                std::process::exit(1);
            }

            print_details(
                &Ip {
                    address: input_ip.expect("nopes"),
                    cidr: input_mask.expect("not cidr"),
                },
                &matches,
                &rows,
                Some(&used),
            );
            std::process::exit(0);
        }

        for line in reader.lines() {
            let ip = parse_address_mask(&line.unwrap());
            if ip.is_none() {
                continue;
            }
            print_details(&ip.unwrap(), &matches, &rows, None);
        }
        std::process::exit(0);
    }

    if input_ip.is_none() {
        println!("{}", opts.usage("ripcalc"));
        eprintln!("Need to provide v4 or v6 address.");
        std::process::exit(1);
    }

    if input_mask.is_none() {
        input_mask = Some(24);
    }

    let num = Ip {
        address: input_ip.expect("nopes"),
        cidr: input_mask.expect("not cidr"),
    };

    match num.address {
        Addr::V4(_) => {
            if num.cidr > 32 {
                eprintln!("V4 mask cannot be greater than 32");
                std::process::exit(1);
            }
        }
        Addr::V6(_) => {
            if num.cidr > 128 {
                eprintln!("V6 mask cannot be greater than 128");
                std::process::exit(1);
            }
        }
    }

    print_details(&num, &matches, &rows, None);
    std::process::exit(0);
}
