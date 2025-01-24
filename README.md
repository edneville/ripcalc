# ripcalc

Calculate or lookup network addresses.

# install

```
$ git clone 'https://gitlab.com/edneville/ripcalc.git'
$ cd ripcalc \
  && cargo build --release \
  && please install -m 0755 -s target/release/ripcalc /usr/local/bin
```

or

```
please snap install ripcalc
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

The same output is visible with stdin:

```
echo '127.0.0.1/8' | ripcalc
...
```

The output format can be customised:

```
$ ripcalc 2001:ba8:1f1:f1cb::4/64 --format "select * from IP6 where (ip >= %ln and ip <= %lb) and active = 1;\nupdate IP6 set active = 0 where (ip >= %ln and ip <= %lb) and active = 1;"
select * from IP6 where (ip >= 42540724579414763292693624807812497408 and ip <= 42540724579414763311140368881522049023) and active = 1;
update IP6 set active = 0 where (ip >= 42540724579414763292693624807812497408 and ip <= 42540724579414763311140368881522049023) and active = 1;
```

# pipes

Sometimes if you want to quickly see if an address is part of a group of networks you can set something like this in your `.bash_aliases`:

```
alias is_our_networks='ripcalc --inside 192.168.0.0/16 --format short'
```

With this alias it would then be possible to do something like this to quickly see if the domain uses your infrastructure:

```
dig +short domain.com | is_our_networks
```

# formatting

*%* denotes a format control character, followed by one of the following:

| placeholder | effect |
|-------------|--------|
| %a          | IP address string |
| %n          | Network address string |
| %s          | Subnet address string |
| %w          | Wildcard address string |
| %b          | Broadcast address string |

Additional characters prefixing the above placeholder can control the representation:

| placeholder | effect |
|-------------|--------|
| %B          | Binary address string |
| %S          | Split binary at network boundary string |
| %l          | Unsigned integer string |
| %L          | Signed integer string |
| %x          | Hex address string |

Other format characters:

| placeholder | effect |
|-------------|--------|
| %c          | CIDR mask |
| %t          | Network size |
| %r          | Network reservation information (if available) |
| %d          | Matching device interface by IP |
| %m          | Matching media link interface by network |
| %p          | PTR record |
| %k          | RBL/reverse DNS-style format |
| %D          | Network size (--networks) |
| %N          | Number of subnets (--networks) |
| %%          | % |
| \n          | Line break |
| \t          | Tab character |

For example:

```
$ ripcalc --format '%k.all.s5h.net\n' 192.168.1.2
2.1.168.192.all.s5h.net
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

When `-a` is used, addresses read from `-s` will not be shown when listing `-l` a network, showing only available addresses.

When `-e` is used with `-s` the smallest encapsulating network will be returned.

Given a list of IP addresses, print only those that match the network. When `s` and `inside` are used, only addresses from `-s` are printed if they are that are inside of the IP source network on the command line. This can be inverted with `--outside:

```
$ echo -e '192.168.0.0\n192.167.255.255\n' | ripcalc -s - --inside 192.168.0.0/16 --format short
192.168.0.0
$ echo -e '192.168.0.0\n192.167.255.255\n' | ripcalc -s - --outside 192.168.0.0/16 --format short
192.167.255.255
```

IP addresses can be treated as reversed, if `/proc/net/route` holds addresses in reversed format, `--reverse inputs` and `--base 16` could be used together to convert to dotted-quad.

# within networks

Is a domain wihtin a list of subnets? For example, in this part of the globe cloudflare.com was being served from their published list of networks:

```
echo http://cloudflare.com | ripcalc --inside 173.245.48.0/20 103.21.244.0/22 \
    103.22.200.0/22 103.31.4.0/22 141.101.64.0/18 108.162.192.0/18 \
    190.93.240.0/20 188.114.96.0/20 197.234.240.0/22 198.41.128.0/17 \
    162.158.0.0/15 104.16.0.0/13 104.24.0.0/14 172.64.0.0/13 \
    131.0.72.0/22 --format short
104.16.133.229
```

How many addresses is all that in total?

```
ripcalc 173.245.48.0/20 103.21.244.0/22 103.22.200.0/22 103.31.4.0/22 \
    141.101.64.0/18 108.162.192.0/18 190.93.240.0/20 188.114.96.0/20 \
    197.234.240.0/22 198.41.128.0/17 162.158.0.0/15 104.16.0.0/13 \
    104.24.0.0/14 172.64.0.0/13 131.0.72.0/22 --format '%t\n' \
    | paste -sd+ \
    | bc -l
1524736
```

If you need to manage a lot of IP addresses this could be helpful to you.

# divide

Networks can be divided into subnets:

```
$ ripcalc 192.168.1.10/24 --divide 26 --format cidr
192.168.1.0/26
192.168.1.64/26
192.168.1.128/26
192.168.1.192/26
```

# quickly block the encapsulating network

Suppose a large flood of requests are from a network pattern, to preserve service you may want to block the whole network that encapsulates a list:

```
please ip route add blackhole `ripcalc -e 192.168.56.10 192.168.57.1 192.168.44.47`
```

Networks can be grouped, in a scenario where you have a list of unwanted traffic, you can turn this into a list of small networks to block, supposing you don't want to block anything that covers more than a /19:

```
cat bad_traffic | ripcalc --encapsulating --group 19 --format cidr
```

# help

```
Options:
    -4, --ipv4 IPv4     ipv4 address
    -6, --ipv6 IPv6     ipv6 address
    -a, --available     display unused addresses
    -b, --base INTEGER  ipv4 base format, default to oct
    -c, --csv PATH      csv reference file
    -d, --divide CIDR   divide network into chunks
        --noexpand      do not expand networks in list
    -e, --encapsulating 
                        display encapsulating network from arguments or lookup
                        list
    -f, --format STRING format output
                        'cidr' expands to %a/%c\n
                        'short' expands to %a\n
                        See manual for more options
        --group CIDR    maximum network group size for encapsulation
    -h, --help          display help
    -i, --field FIELD   csv field
    -l, --list          list all addresses in network
        --outside       display when extremities are outside network
        --inside        display when extremities are inside network
    -m, --mask CIDR     cidr mask
    -n, --networks CIDR instead of hosts, display number of subnets of this
                        size
    -r, --reverse       (none, inputs, sources or both) v4 octets, v6 hex
    -s, --file PATH     lookup addresses from, - for stdin
    -v, --version       print version
```


