# ripcalc

Calculate or lookup network addresses.

# install

```
$ git clone 'https://gitlab.com/edneville/ripcalc.git'
$ cd ripcalc \
  && cargo build --release \
  && please install -m 0755 -s target/release/ripcalc /usr/local/bin
```

# usage

Ripcalc allows networks to be provided by argument

```
$ ripcalc 127.0.0.1/8
IP is: 127.0.0.1/8
Unsigned IP: 2130706433
Broadcast is: 127.255.255.255
Unsigned broadcast: 2147483647
Network is: 127.0.0.0
Unsigned network: 2130706432
Subnet is: 255.0.0.0
Unsigned subnet: 4278190080
Wildcard is: 0.255.255.255
Unsigned wildcard: 16777215
Network size: 16777216
```

The output format can be customised:

```
$ ripcalc 2001:ba8:1f1:f1cb::4/64 --format "select * from IP6 where (ip >= %ln and ip <= %lb) and active = 1;\nupdate IP6 set active = 0 where (ip >= %ln and ip <= %lb) and active = 1;"
select * from IP6 where (ip >= 42540724579414763292693624807812497408 and ip <= 42540724579414763311140368881522049023) and active = 1;
update IP6 set active = 0 where (ip >= 42540724579414763292693624807812497408 and ip <= 42540724579414763311140368881522049023) and active = 1;
```

With a csv it can find networks that an IP address is within, use `%{field}` to print matches:

```
$ cat nets.csv
network,range,owner
rfc1918,192.168.0.0/16,bob
rfc1918,172.16.0.0/12,cliff
rfc1918,10.0.0.0/8,mr nobody

$ ripcalc --csv nets.csv -i range --format '%{owner}\n' 192.168.0.0
bob
```

```
Options:
    -4, --ipv4 IPv4     ipv4 address
    -6, --ipv6 IPv6     ipv6 address
    -f, --format STRING format output
    -m, --mask CIDR     cidr mask
    -c, --csv PATH      csv reference file
    -i, --field FIELD   csv field
    -h, --help          display help
    -s, --file PATH     lookup addresses from, - for stdin
```


