use getopts::Options;
use ripcalc::*;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::os::unix::io::AsRawFd;
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
        let width = 25;
        match ip.address {
            Addr::V4(_) => format!("{ip:>width$}/{cidr}\n{broadcast:>width$}\n{network:>width$}\n{subnet:>width$}\n{wildcard:>width$}\n{network_size:>width$}\n", ip="IP is: %a", cidr="%c", broadcast="Broadcast is: %b", network="Network is: %n", subnet="Subnet is: %s", wildcard="Wildcard is: %w", network_size="Network size: %t", width=width),
            Addr::V6(_) => format!("{ip:>widthn$}/{cidr}\n{expanded:>width$}\n{network:>width$}\n{last_host_address:>width$}\n{subnet:>width$}\n{network_size:>widthn$}\n", ip="IP is: %a", cidr="%c", expanded="Expanded: %xa", network="Network is: %xn", last_host_address="Last host address: %xb", subnet="Subnet is: %xs", network_size="Network size: %t", width=width, widthn=width-1),
        }
    };

    if formatted == "cidr" {
        formatted = "%a/%c\n".to_string();
    }

    if formatted == "short" {
        formatted = "%a\n".to_string();
    }

    if matches.opt_present("divide") {
        let divide: u32 = match matches.opt_str("divide").unwrap().trim().parse() {
            Ok(x) => x,
            Err(x) => {
                eprintln!("Cannot convert {} to number", x);
                std::process::exit(1);
            }
        };

        for ip_copy in addresses(ip, used, Some(divide)) {
            if let Some(m) = format_details(&ip_copy, formatted.to_string(), rows) {
                print!("{}", m);
            }
        }
        return;
    }

    if matches.opt_present("list") {
        for ip_copy in addresses(ip, used, None) {
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

fn banner() -> String {
    format!(
        "{} version {}",
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION")
    )
}

fn print_version() {
    println!("{}", &banner());
}

fn process_csv(
    mut reader: csv::Reader<File>,
    field_name: String,
    rows: &mut Option<HashMap<Ip, NetRow>>,
    input_base: Option<i32>,
    reverse: &Reverse,
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

                let v6 = parse_v6(parts[0], matches!(reverse, Reverse::Both | Reverse::Source));

                if v6.is_none() {
                    eprintln!("{}: not in ip/cidr format", rec);
                    continue;
                }
                row_ip = v6;
            }

            if parts[0].contains('.') {
                if rows.is_none() {
                    *rows = Some(HashMap::new());
                }

                let v4 = parse_v4(
                    parts[0],
                    input_base,
                    matches!(reverse, Reverse::Both | Reverse::Source),
                );

                if v4.is_none() {
                    eprintln!("{}: not in ip/cidr format", rec);
                    continue;
                }
                row_ip = v4;
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

fn process_input_file(
    path: &str,
    matches: &getopts::Matches,
    input_base: Option<i32>,
    reverse: &Reverse,
    ip_args: &[Ip],
    rows: &Option<HashMap<Ip, NetRow>>,
    inside: Option<bool>,
) {
    let reader: Box<dyn BufRead> = if path == "-" {
        Box::new(BufReader::new(std::io::stdin()))
    } else {
        let path = std::path::Path::new(&path);
        if !path.exists() {
            println!(
                "Could not open {} as it does not exist",
                path.to_string_lossy()
            );
            std::process::exit(1);
        }
        Box::new(BufReader::new(File::open(path).unwrap()))
    };

    let mut used: HashMap<Addr, bool> = HashMap::new();
    if matches.opt_present("available") {
        for line in reader.lines() {
            let ip = match parse_address_mask(
                line.as_ref().unwrap(),
                Some(32),
                Some(128),
                input_base,
                matches!(reverse, Reverse::Both | Reverse::Input),
            ) {
                Some(x) => x,
                None => {
                    eprintln!("Could not parse {}", &line.as_ref().unwrap());
                    continue;
                }
            };
            for ip in addresses(&ip, None, None) {
                used.insert(ip.address, true);
            }
        }
        for arg in ip_args {
            print_details(arg, matches, rows, Some(&used));
        }
        std::process::exit(0);
    }

    if matches.opt_present("e") {
        let mut used: HashMap<Ip, bool> = HashMap::new();
        for line in reader.lines() {
            let ip = match parse_address_mask(
                line.as_ref().unwrap(),
                Some(32),
                Some(128),
                input_base,
                matches!(reverse, Reverse::Both | Reverse::Input),
            ) {
                Some(x) => x,
                None => {
                    eprintln!("Could not parse {}", &line.as_ref().unwrap());
                    continue;
                }
            };
            used.insert(ip, true);
        }

        match smallest_group_network(&used) {
            Some(x) => {
                print_details(&x, matches, rows, None);
            }
            None => {
                eprintln!("Could not find an encapsulating network, sorry");
                std::process::exit(1);
            }
        }

        std::process::exit(0);
    }

    for line in reader.lines() {
        let ip = parse_address_mask(
            &line.unwrap(),
            Some(32),
            Some(128),
            input_base,
            matches!(reverse, Reverse::Both | Reverse::Source),
        );
        if ip.is_none() {
            continue;
        }

        match inside {
            Some(true) => {
                let mut found = false;
                for arg in ip_args {
                    if within(arg, &ip.as_ref().unwrap().address) {
                        found = true;
                        break;
                    }
                }

                if found {
                    print_details(&ip.unwrap(), matches, rows, None);
                }
            }
            Some(false) => {
                let mut found = false;

                for arg in ip_args {
                    if within(arg, &ip.as_ref().unwrap().address) {
                        found = true;
                        break;
                    }
                }

                if !found {
                    print_details(&ip.unwrap(), matches, rows, None);
                }
            }
            None => {
                print_details(&ip.unwrap(), matches, rows, None);
            }
        }
    }
}

fn wait_stdin(matches: &getopts::Matches) -> bool {
    if matches.opt_present("available") {
        return true;
    }
    if matches.opt_present("list")
        && (matches.opt_present("inside") || matches.opt_present("outside"))
    {
        return true;
    }
    if matches.free.is_empty() && matches.opt_present("divide") {
        return true;
    }
    false
}

fn main() {
    let mut opts = Options::new();
    let mut rows: Option<HashMap<Ip, NetRow>> = None;
    let mut input_ip: Option<Addr> = None;
    let mut input_mask: Option<u32> = None;
    let mut input_base: Option<i32> = None;
    let mut reverse = Reverse::None;
    let mut inside: Option<bool> = None;
    let args: Vec<String> = std::env::args().collect();
    let mut ip_args: Vec<Ip> = vec![];
    opts.parsing_style(getopts::ParsingStyle::FloatingFrees);
    opts.optopt("4", "ipv4", "ipv4 address", "IPv4");
    opts.optopt("6", "ipv6", "ipv6 address", "IPv6");

    opts.optflag("a", "available", "display unused addresses");
    opts.optopt("b", "base", "ipv4 base format, default to oct", "INTEGER");
    opts.optopt("c", "csv", "csv reference file", "PATH");
    opts.optopt("d", "divide", "divide network into chunks", "CIDR");

    opts.optflag(
        "e",
        "encapsulating",
        "display encapsulating network from lookup list",
    );

    opts.optopt("f", "format", "format output\n'cidr' expands to %a/%c\\n\n'short' expands to %a\\n\nSee manual for more options", "STRING");
    opts.optflag("h", "help", "display help");

    opts.optopt("i", "field", "csv field", "FIELD");
    opts.optflag("l", "list", "list all addresses in network");
    opts.optflag("", "outside", "display only outside network");
    opts.optflag("", "inside", "display only inside network");
    opts.optopt("m", "mask", "cidr mask", "CIDR");

    opts.optopt(
        "r",
        "reverse",
        "(none, inputs, sources or both) v4 octets, v6 hex",
        "",
    );
    opts.optopt("s", "file", "lookup addresses from, - for stdin", "PATH");

    opts.optflag("v", "version", "print version");

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(f) => {
            println!("{}", f);
            std::process::exit(1);
        }
    };

    if matches.opt_present("h") {
        println!("{}", opts.usage(&banner()));

        std::process::exit(0);
    }

    if matches.opt_present("version") {
        print_version();
        std::process::exit(0);
    }

    if matches.opt_present("inside") {
        inside = Some(true);
    }

    if matches.opt_present("outside") {
        if inside.is_some() {
            println!("Cannot combine --inside and --outside");
            std::process::exit(1);
        }
        inside = Some(false);
    }

    if matches.opt_present("reverse") {
        match matches.opt_str("reverse").unwrap().as_str() {
            "inputs" => {
                reverse = Reverse::Input;
            }
            "sources" => {
                reverse = Reverse::Source;
            }
            "both" => {
                reverse = Reverse::Both;
            }
            _ => {
                println!("reverse is not one of inputs, sources or both");
                std::process::exit(1);
            }
        }
    }

    if matches.opt_present("base") {
        input_base = match i32::from_str(&matches.opt_str("b").unwrap()) {
            Ok(x) => Some(x),
            Err(x) => {
                println!("Cannot convert to an integer base: {}", x);
                std::process::exit(1);
            }
        };
    }

    if matches.opt_present("csv") {
        let path = matches.opt_str("csv").unwrap();
        let reader = match csv::Reader::from_path(&path) {
            Ok(r) => r,
            Err(x) => {
                eprintln!("Cannot open {}: {}", &path, x);
                std::process::exit(1);
            }
        };
        let field_name = if matches.opt_present("field") {
            matches.opt_str("field").unwrap()
        } else {
            "network".to_string()
        };

        process_csv(reader, field_name, &mut rows, input_base, &reverse);
    }

    if let Some(v) = matches.opt_str("mask") {
        input_mask = parse_mask(&v);
    }

    if let Some(v) = matches.opt_str("ipv4") {
        input_ip = parse_v4(
            &v,
            input_base,
            matches!(reverse, Reverse::Both | Reverse::Input),
        );
    }

    if let Some(v) = matches.opt_str("ipv6") {
        input_ip = parse_v6(&v, matches!(reverse, Reverse::Both | Reverse::Input));
    }

    if input_mask.is_none() && matches.free.is_empty() {
        input_mask = Some(24);

        if let Some(Addr::V6(_)) = input_ip {
            input_mask = Some(64);
        }
    }

    if let Some(input_ip) = input_ip {
        ip_args.push(Ip {
            address: input_ip,
            cidr: input_mask.unwrap(),
        });
    }

    let free_arg = matches.free.clone();
    if !free_arg.is_empty() {
        for arg in &free_arg {
            let ip = parse_address_mask(
                arg,
                input_mask,
                input_mask,
                input_base,
                matches!(reverse, Reverse::Both | Reverse::Input),
            );

            if let Some(ip) = ip {
                ip_args.push(ip);
            }
        }
    }

    let stdin_ready = fd_ready(std::io::stdin().as_raw_fd());
    if (stdin_ready && wait_stdin(&matches)) || matches.opt_str("file").is_some() {
        let path = if stdin_ready {
            "-".to_string()
        } else {
            matches.opt_str("file").unwrap()
        };
        process_input_file(
            &path, &matches, input_base, &reverse, &ip_args, &rows, inside,
        );

        if ip_args.clone().is_empty() || inside.is_some() {
            std::process::exit(0);
        }
    }

    for arg in &ip_args {
        match arg.address {
            Addr::V4(_) => {
                if arg.cidr > 32 {
                    eprintln!("V4 mask cannot be greater than 32");
                    std::process::exit(1);
                }
            }
            Addr::V6(_) => {
                if arg.cidr > 128 {
                    eprintln!("V6 mask cannot be greater than 128");
                    std::process::exit(1);
                }
            }
        }

        print_details(arg, &matches, &rows, None);
    }

    if ip_args.is_empty() {
        println!("{}", opts.usage("ripcalc"));
        eprintln!("Need to provide v4 or v6 address.");
        std::process::exit(1);
    }

    std::process::exit(0);
}
