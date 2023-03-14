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
Broadcast is: 127.255.255.255
Network is: 127.0.0.0
Subnet is: 255.0.0.0
Wildcard is: 0.255.255.255
Network size: 16777216
```

The output format can be customised:

```
$ ripcalc 2001:ba8:1f1:f1cb::4/64 --format "select * from IP6 where (ip >= %ln and ip <= %lb) and active = 1;\nupdate IP6 set active = 0 where (ip >= %ln and ip <= %lb) and active = 1;"
select * from IP6 where (ip >= 42540724579414763292693624807812497408 and ip <= 42540724579414763311140368881522049023) and active = 1;
update IP6 set active = 0 where (ip >= 42540724579414763292693624807812497408 and ip <= 42540724579414763311140368881522049023) and active = 1;
```

| placeholder | effect |
|-------------|--------|
| %a          | IP address string |
| %xa         | IP address in hex quad |
| %c          | cidr mask |
| %la         | IP in unsigned numeric |
| %b          | broadcast in string format |
| %xb         | broadcast in hex quad |
| %lb         | broadcast in numeric |
| %n          | network in string format |
| %xn         | network in hex quad |
| %ln         | network in numeric |
| %s          | subnet in string format |
| %xs         | subnet in hex quad |
| %ls         | subnet in numeric |
| %w          | wildcard in string format |
| %xw         | wildcard in hex quad |
| %lw         | wildcard in numeric |
| %t          | network size |
| %r          | network reservation information (if available) |

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

Addresses can be read via file or from stdin (-):

```
$ cat list
127.0.0.1/28
10.0.0.1/28
192.168.1.1/30
172.18.1.1/30
10.0.0.0/30
$ ripcalc --csv nets.csv -i range --format '%{range} %{owner}\n' -s list
10.0.0.0/8 mr nobody
192.168.0.0/16 bob
172.16.0.0/12 cliff
10.0.0.0/8 mr nobody
```

Options:

```
    -4, --ipv4 IPv4     ipv4 address
    -6, --ipv6 IPv6     ipv6 address
    -f, --format STRING format output
    -m, --mask CIDR     cidr mask
    -c, --csv PATH      csv reference file
    -i, --field FIELD   csv field
    -l, --list          list all addresses in network
    -h, --help          display help
    -s, --file PATH     lookup addresses from, - for stdin
```


