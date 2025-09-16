use getopts::Options;
use reqwest::header::HeaderMap;
use ripcalc::*;
use serde_json::*;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::os::fd::AsFd;
use std::str::FromStr;

fn print_details(
    ip: &Ip,
    matches: &getopts::Matches,
    rows: &Option<HashMap<Ip, NetRow>>,
    used: Option<&HashMap<Addr, bool>>,
    config: &RefCell<Config>,
) {
    let mut networks: Option<u32> = None;

    if matches.opt_present("networks") {
        let nets = matches.opt_str("networks").unwrap().trim().parse().unwrap();

        if nets < ip.cidr && !(matches.opt_present("encapsulating") && matches.opt_present("group"))
        {
            eprintln!("{} is bigger than the network mask {}", nets, ip.cidr);
            std::process::exit(1);
        }

        match ip.address {
            Addr::V4(_) => {
                if nets > 32 {
                    eprintln!("{} is too big", nets);
                    std::process::exit(1);
                }
            }
            Addr::V6(_) => {
                if nets > 128 {
                    eprintln!("{} is too big", nets);
                    std::process::exit(1);
                }
            }
        }
        networks = Some(nets);
    }

    let mut formatted = if matches.opt_present("f") {
        matches.opt_str("f").unwrap()
    } else {
        let mut network_size = "Network size: %t".to_string();
        let width = 25;
        if matches.opt_present("networks") {
            network_size = format!("Networks ({}): %N", networks.as_ref().unwrap()).to_string();
        }
        match ip.address {

            Addr::V4(_) => format!("{ip:>width$}/{cidr}\n{broadcast:>width$}\n{network:>width$}\n{subnet:>width$}\n{wildcard:>width$}\n{network_size:>width$}\n", ip="IP is: %a", cidr="%c", broadcast="Broadcast is: %b", network="Network is: %n", subnet="Subnet is: %s", wildcard="Wildcard is: %w", network_size=network_size, width=width),
            Addr::V6(_) => format!("{ip:>widthn$}/{cidr}\n{expanded:>width$}\n{network:>width$}\n{last_host_address:>width$}\n{subnet:>width$}\n{network_size:>widthn$}\n", ip="IP is: %a", cidr="%c", expanded="Expanded: %xa", network="Network is: %xn", last_host_address="Last host address: %xb", subnet="Subnet is: %xs", network_size=network_size, width=width, widthn=width-1),
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
            if let Some(m) = format_details(
                &ip_copy,
                formatted.to_string(),
                rows,
                networks,
                Some(matches),
                config,
            ) {
                print!("{}", m);
            }
        }
        return;
    }

    if matches.opt_present("list") {
        if matches.opt_present("noexpand") {
            if let Some(m) = format_details(ip, formatted, rows, networks, Some(matches), config) {
                print!("{}", m);
            }
            return;
        }

        for ip_copy in addresses(ip, used, None) {
            if let Some(m) = format_details(
                &ip_copy,
                formatted.to_string(),
                rows,
                networks,
                Some(matches),
                config,
            ) {
                print!("{}", m);
            }
        }
        return;
    }

    if config_option_true(&config.borrow(), "abuseipdb".to_string()) {
        let cfg = config.borrow();

        let key = cfg.options.get("abuseipdb");
        let key = key.unwrap().clone();
        drop(cfg);

        get_abuseipdb_details(config, ip, &key);
    }

    if let Some(m) = format_details(ip, formatted, rows, networks, Some(matches), config) {
        print!("{}", m);
    }
}

fn get_abuseipdb_details(config: &RefCell<Config>, ip: &Ip, key: &str) {
    let mut headers = HeaderMap::new();
    headers.insert("Key", key.parse().unwrap());

    let client = reqwest::blocking::Client::new();
    let response = client
        .get(format!(
            "https://api.abuseipdb.com/api/v2/check?ipAddress={}&maxAgeInDays={}",
            ip, 30
        ))
        .header("Key".to_string(), key)
        .send();

    let body = response.unwrap().text().unwrap();
    let h: Value = serde_json::from_str(&body).unwrap();

    let mut config = config.borrow_mut();
    if h["data"].as_object().is_none() {
        eprintln!("Error communicating with abuseipdb.com");
        std::process::exit(1);
    }

    let data = h["data"].as_object().unwrap();
    for k in data.keys() {
        config
            .options
            .insert(format!("abuseipdb_{}", k).to_string(), data[k].to_string());
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

fn increment_seen_count(config: &mut Config, i: Ip) {
    if config_option_true(config, "countseen".to_string()) && config.count.is_some() {
        let count = config.count.as_ref().unwrap().get(&i).unwrap_or(&0) + 1;
        config.count.as_mut().unwrap().insert(i.clone(), count);
    }
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
            let mut parts: Vec<&str> = rec.split('/').collect();
            let mut row_ip: Option<Addr> = None;

            if parts[0].contains(':') {
                if rows.is_none() {
                    *rows = Some(HashMap::new());
                }

                if parts.len() == 1 {
                    parts.push("128");
                }

                let v6 = parse_v6(
                    parts[0],
                    input_base,
                    matches!(reverse, Reverse::Both | Reverse::Source),
                );

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
                if parts.len() == 1 {
                    parts.push("32");
                }

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

fn process_encapsulated_input(
    matches: &getopts::Matches,
    inside: Option<bool>,
    ip_args: &[Ip],
    i: &Ip,
    network_size: u32,
    used: &mut HashMap<Ip, bool>,
    config: &RefCell<Config>,
) {
    if !inside_filter(inside, ip_args, i) {
        return;
    }
    let mut network_size = network_size;

    used.insert(i.clone(), true);

    increment_seen_count(&mut config.borrow_mut(), i.clone());

    if matches.opt_present("group") {
        match i.address {
            Addr::V4(_) => {
                if network_size > 32 {
                    network_size = 32;
                }
            }
            Addr::V6(_) => {
                if network_size > 128 {
                    network_size = 128;
                }
            }
        }
        let net_mask = network(&Ip {
            address: i.address.clone(),
            cidr: network_size,
        });
        let nu = &mut config.borrow_mut().net_used;

        if nu.get(&net_mask).is_none() {
            nu.insert(net_mask.clone(), HashMap::new());
        }

        let nm = nu.get_mut(&net_mask).unwrap();
        nm.insert(i.clone(), true);
    }
}

fn process_group_encapsulation(
    matches: &getopts::Matches,
    used: HashMap<Ip, bool>,
    rows: &Option<HashMap<Ip, NetRow>>,
    config: &RefCell<Config>,
) {
    if matches.opt_present("group") {
        let network_size: u32 = matches.opt_str("group").unwrap().trim().parse().unwrap();
        match smallest_group_network_limited(&used, network_size) {
            Some(mut x) => {
                x.sort_by(|a, b| a.partial_cmp(b).unwrap());
                for y in x {
                    {
                        let net_mask = network(&Ip {
                            address: y.address.clone(),
                            cidr: network_size,
                        });

                        let cfg = &mut config.borrow_mut();
                        if !cfg.net_used.contains_key(&net_mask) {
                            cfg.net_used.insert(net_mask.clone(), HashMap::new());
                        }

                        let nm = cfg.net_used.get(&net_mask).unwrap();

                        cfg.used = Some(nm.clone());
                    }

                    print_details(&y, matches, rows, None, config);
                }
            }
            None => {
                eprintln!("Could not find an encapsulating network, sorry");
                std::process::exit(1);
            }
        }
    } else {
        match smallest_group_network(&used) {
            Some(x) => {
                print_details(&x, matches, rows, None, config);
            }
            None => {
                eprintln!("Could not find an encapsulating network, sorry");
                std::process::exit(1);
            }
        }
    }
}

fn process_input_filter(
    reader: &mut Box<dyn BufRead>,
    ip_args: &[Ip],
    inside: Option<bool>,
    config: &RefCell<Config>,
) {
    let position: u32 = match config
        .borrow()
        .options
        .get("filter")
        .unwrap()
        .trim()
        .parse()
    {
        Ok(x) => {
            if x < 1 {
                eprintln!("filternum {} should be 1 or more", x);
                std::process::exit(1);
            }
            x
        }
        Err(x) => {
            eprintln!("Cannot convert {} to number", x);
            std::process::exit(1);
        }
    };

    for line in reader.lines() {
        match line {
            Ok(_) => {
                let line = line.unwrap();
                let parts: Vec<&str> = line.split(' ').collect();

                if parts.is_empty() || parts.len() < position as usize {
                    continue;
                }

                if let Some(ip) = parse_address_mask(
                    parts[(position - 1) as usize],
                    None,
                    None,
                    None,
                    false,
                    config,
                ) {
                    if inside_filter(Some(inside.unwrap_or(true)), ip_args, &ip) {
                        println!("{}", line);
                    }
                }
            }
            Err(x) => {
                eprintln!("Cannot read input: {}", x);
                std::process::exit(1);
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn process_input_file(
    path: &str,
    matches: &getopts::Matches,
    input_base: Option<i32>,
    reverse: &Reverse,
    ip_args: &[Ip],
    rows: &Option<HashMap<Ip, NetRow>>,
    inside: Option<bool>,
    config: &RefCell<Config>,
) {
    let mut reader: Box<dyn BufRead> = if path == "-" {
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

    if matches.opt_present("filter") || matches.opt_present("filternum") {
        process_input_filter(&mut reader, ip_args, inside, config);

        std::process::exit(0);
    }

    if matches.opt_present("available") {
        let mut used: HashMap<Addr, bool> = HashMap::new();
        for a in find_ips(&mut reader, input_base, reverse, config) {
            for ip in a {
                used.insert(ip.address, true);
            }
        }

        for arg in ip_args {
            print_details(arg, matches, rows, Some(&used), config);
        }
        std::process::exit(0);
    }

    if matches.opt_present("encapsulating") {
        let mut used: HashMap<Ip, bool> = HashMap::new();
        let mut network_size: u32 = 0;

        if matches.opt_present("group") {
            network_size = matches.opt_str("group").unwrap().trim().parse().unwrap();
        }

        for a in find_ips(&mut reader, input_base, reverse, config) {
            for i in a {
                process_encapsulated_input(
                    matches,
                    inside,
                    ip_args,
                    &i,
                    network_size,
                    &mut used,
                    config,
                );
            }
        }

        config.borrow_mut().used = Some(used.clone());

        process_group_encapsulation(matches, used, rows, config);

        std::process::exit(0);
    }

    let mut found_match = false;

    for a in find_ips(&mut reader, input_base, reverse, config) {
        for ip in a {
            increment_seen_count(&mut config.borrow_mut(), ip.clone());
            if inside_filter(inside, ip_args, &ip) {
                found_match = true;
                print_details(&ip, matches, rows, None, config);
            }
        }
    }

    if !found_match && inside.is_some() {
        std::process::exit(1);
    }
}

fn wait_stdin(matches: &getopts::Matches) -> bool {
    if matches.opt_present("available") {
        return true;
    }
    if matches.opt_present("list")
        || matches.opt_present("inside")
        || matches.opt_present("outside")
        || matches.opt_present("filter")
        || matches.opt_present("filternum")
    {
        return true;
    }
    if matches.free.is_empty() {
        return true;
    }
    false
}

fn make_cdb(path: String, config: &RefCell<Config>) {
    let reader = BufReader::new(std::io::stdin());
    let tmp_file = format!("{}.tmp", path);
    let cdb = cdb::CDBWriter::create(&tmp_file);

    if cdb.is_err() {
        eprintln!("Cannot create: {}", tmp_file);
        std::process::exit(1);
    }

    let mut cdb = cdb.unwrap();

    for line in reader.lines() {
        match line {
            Ok(_) => {
                let line = line.unwrap();
                let mut parts: Vec<&str> = line.split(',').collect();
                if let Some(key) = parse_address_mask(parts[0], None, None, None, false, config) {
                    for j in &mut parts {
                        *j = j.trim();
                    }
                    let key = format!("{}/{}", network(&key), key.cidr);
                    let value = parts[1..].join("\0");
                    let _ = cdb.add(key.as_bytes(), value.as_bytes());
                }
            }
            Err(x) => {
                eprintln!("Cannot read: {}", x);
                std::process::exit(1);
            }
        }
    }

    if cdb.finish().is_err() {
        eprintln!("Cannot finish writing: {}", tmp_file);
        std::process::exit(1);
    }

    if fs::rename(&tmp_file, &path).is_err() {
        eprintln!("Cannot rename {} to {}", tmp_file, path);
        std::process::exit(1);
    }
}

fn main() {
    let mut opts = Options::new();
    let mut rows: Option<HashMap<Ip, NetRow>> = None;
    let input_ip: Option<Addr> = None;
    let mut input_mask: Option<u32> = None;
    let mut input_base: Option<i32> = None;
    let mut reverse = Reverse::None;
    let mut inside: Option<bool> = None;
    let args: Vec<String> = std::env::args().collect();
    let mut ip_args: Vec<Ip> = vec![];
    let config = RefCell::new(Config::new());

    opts.parsing_style(getopts::ParsingStyle::FloatingFrees);
    opts.optflag("4", "ipv4", "treat inputs as ipv4 address");
    opts.optflag("6", "ipv6", "treat inputs as ipv6 address");

    opts.optflag("a", "available", "display unused addresses");
    opts.optopt("", "abuseipdb", "abuseipdb API", "KEY");
    opts.optflag(
        "",
        "allowemptyrow",
        "when no matching csv network, use empty fields",
    );
    opts.optopt("b", "base", "ipv4 base format, default to oct", "INTEGER");
    opts.optopt("", "cdb", "cdb reference file", "PATH");
    opts.optflag("", "countseen", "count times an ip is seen");
    opts.optopt("c", "csv", "csv reference file", "PATH");
    opts.optopt("d", "divide", "divide network into chunks", "CIDR");
    opts.optflag("", "noexpand", "do not expand networks in list");

    opts.optflag(
        "e",
        "encapsulating",
        "display encapsulating network from arguments or lookup list",
    );

    opts.optopt("f", "format", "format output\n'cidr' expands to %a/%c\\n\n'short' expands to %a\\n\nSee manual for more options", "STRING");
    opts.optflag("", "filter", "print STDIN line if field parses as an IP and matches, defaults to 1, use filternum to adjust");
    opts.optopt(
        "",
        "filternum",
        "change position NUMBER for IP filter, enables --filter",
        "NUMBER",
    );
    opts.optopt(
        "",
        "group",
        "maximum network group size for encapsulation",
        "CIDR",
    );
    opts.optflag("h", "help", "display help");

    opts.optopt("i", "field", "csv/db field", "FIELD");
    opts.optflag("l", "list", "list all addresses in network");
    opts.optflag(
        "",
        "outside",
        "display when extremities are outside network",
    );
    opts.optflag("", "inside", "display when extremities are inside network");
    opts.optopt("", "makecdb", "build a cdb file from STDIN", "PATH");
    opts.optopt("m", "mask", "cidr mask", "CIDR");
    opts.optopt(
        "n",
        "networks",
        "instead of hosts, display number of subnets of this size",
        "CIDR",
    );

    opts.optflag("q", "quiet", "don't report errors");

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

    if matches.opt_present("quiet") {
        config
            .borrow_mut()
            .options
            .insert("quiet".to_string(), "true".to_string());
    }

    if matches.opt_present("group") {
        let _: u32 = match matches.opt_str("group").unwrap().trim().parse() {
            Ok(x) => {
                if x > 128 {
                    eprintln!("Cannot use a group greater than 128");
                    std::process::exit(1);
                }
                x
            }
            Err(x) => {
                eprintln!("Cannot convert {} to number", x);
                std::process::exit(1);
            }
        };
    }

    if matches.opt_present("networks") {
        let _: u32 = match matches.opt_str("networks").unwrap().trim().parse() {
            Ok(x) => x,
            Err(x) => {
                eprintln!("Cannot convert {} to number", x);
                std::process::exit(1);
            }
        };
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
        input_base = match i32::from_str(&matches.opt_str("base").unwrap()) {
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

    if matches.opt_present("cdb") {
        let path = matches.opt_str("cdb").unwrap();

        config
            .borrow_mut()
            .options
            .insert("cdb_path".to_string(), path);
    }

    if matches.opt_present("countseen") {
        config
            .borrow_mut()
            .options
            .insert("countseen".to_string(), "true".to_string());
        config.borrow_mut().count = Some(HashMap::new());
    }

    if let Some(v) = matches.opt_str("mask") {
        input_mask = parse_mask(&v);
    }

    if let Some(v) = matches.opt_str("makecdb") {
        make_cdb(v, &config);
        std::process::exit(0);
    }

    if matches.opt_present("ipv4") {
        config.borrow_mut().input_family = Some(InputFamily::IPv4);
    }

    if matches.opt_present("ipv6") {
        config.borrow_mut().input_family = Some(InputFamily::IPv6);
    }

    if matches.opt_present("filter") {
        config
            .borrow_mut()
            .options
            .insert("filter".to_string(), String::from("1"));
    }

    if matches.opt_present("abuseipdb") {
        config.borrow_mut().options.insert(
            "abuseipdb".to_string(),
            matches.opt_str("abuseipdb").unwrap(),
        );
    }

    if matches.opt_present("filternum") {
        let num: u32 = match matches.opt_str("filternum").unwrap().trim().parse() {
            Ok(x) => x,
            Err(x) => {
                eprintln!("Cannot convert {} to number", x);
                std::process::exit(1);
            }
        };

        config
            .borrow_mut()
            .options
            .insert("filter".to_string(), num.to_string());
    }

    if input_mask.is_none() && matches.free.is_empty() {
        input_mask = Some(32);

        if let Some(Addr::V6(_)) = input_ip {
            input_mask = Some(128);
        }
    }

    if let Some(input_ip) = input_ip {
        ip_args.push(Ip {
            address: input_ip,
            cidr: input_mask.unwrap(),
        });
    }

    if !matches.opt_present("inside")
        && !matches.opt_present("outside")
        && matches.opt_present("encapsulating")
        && !matches.opt_present("mask")
    {
        input_mask = Some(32);
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
                &config,
            );

            if let Some(ip) = ip {
                ip_args.push(ip);
            }
        }
    }

    let stdin_ready = fd_ready(std::io::stdin().as_fd());
    if (stdin_ready && wait_stdin(&matches)) || matches.opt_str("file").is_some() {
        let path = if stdin_ready {
            "-".to_string()
        } else {
            matches.opt_str("file").unwrap()
        };
        process_input_file(
            &path, &matches, input_base, &reverse, &ip_args, &rows, inside, &config,
        );

        if ip_args.clone().is_empty() || inside.is_some() {
            std::process::exit(0);
        }
    }

    let mut used: HashMap<Ip, bool> = HashMap::new();

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

        if matches.opt_present("encapsulating") {
            let mut network_size: u32 = 0;

            if matches.opt_present("group") {
                network_size = matches.opt_str("group").unwrap().trim().parse().unwrap();
            }

            process_encapsulated_input(
                &matches,
                inside,
                &ip_args,
                arg,
                network_size,
                &mut used,
                &config,
            );
            used.insert(arg.clone(), true);
            continue;
        }
        print_details(arg, &matches, &rows, None, &config);
    }

    config.borrow_mut().used = Some(used.clone());

    if matches.opt_present("encapsulating") {
        process_group_encapsulation(&matches, used, &rows, &config);
        std::process::exit(0);
    }

    if ip_args.is_empty() {
        println!("{}", opts.usage("ripcalc"));
        eprintln!("Need to provide v4 or v6 address.");
        std::process::exit(1);
    }

    std::process::exit(0);
}
