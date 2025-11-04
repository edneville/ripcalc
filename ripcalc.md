---
title: ripcalc
section: 1
header: User Manual
footer: ripcalc 0.2.7
author: Ed Neville (ed-ripcalc@s5h.net)
date: 04 November 2025
---

# NAME

ripcalc - a tool for network addresses

# SYNOPSIS

**ripcalc 127.0.0.1**

**ripcalc -4/--ipv4 127.0.0.1**

**ripcalc -6/--ipv6 ::1**

**ripcalc -f/--format "%a/%c\n" 127.0.0.1**

**ripcalc -m/--mask 28 127.0.0.1**

**ripcalc -c/--csv path/to/csv [-i/--field network] 127.0.0.1**

**ripcalc -l/--list 127.0.0.1**

**ripcalc -a/--available**

**ripcalc --abuseipdb [KEY]**

**ripcalc -s/--file [-] 127.0.0.1**

**ripcalc -e/--encapsulating [-s/--file name] [--group CIDR] [--countseen]**

**ripcalc -s/--file name [--inside/--outside] 127.0.0.1**

**ripcalc -b/--base [8, 10, 16 etc]**

**ripcalc -d/--divide [CIDR] 127.0.0.1/24**

**ripcalc --makecdb [PATH]**

**ripcalc --makethymecdb [PATH]**

**ripcalc --networks [CIDR] 127.0.0.1/24**

**ripcalc --filter [CIDR]**

**ripcalc --filternum [NUMBER] [CIDR]**

**ripcalc -q/--quiet**

**ripcalc -h/--help**

# DESCRIPTION

**ripcalc** can read IPv4/IPv6 addresses from command line or standard input and output different formats or associated networks from **CSV**.

**ripcalc** can format network addresses, find matches in **CSV** or process a list.

**ripcalc** can convert input addresses that are in other number formats such as hex or octal.

Given a list of IP addresses, print only those that match the network. When `s` and `inside` are used, only addresses from `-s` are printed if they are that are inside of the input IP network from the command line. This can be reversed with `--outside`, (e.g. `ripcalc -s - --inside 192.168.0.0/16`).

When `-a` is used, addresses read from `-s` will not be shown when listing `-l` a network, showing only available addresses.

When `--reverse` is used the `inputs`, `sources` or both can be treated as back-to-front.

**ripcalc** can return a list of subnets when a network is provided along with the `--divide` argument and a subnet CIDR mask.

When `--encapsulating` is used the containing network will be returned, use with `--group` to limit the range that an encapsulating network can grow. Combine with `--countseen` to count the number of times the addresses are read in.

The number (**%D**) of subnets can be printed when using the `--group` argument with the **%N** formatters. The argument should be the CIDR mask, see below for example.

If **base** is a negative number, input addresses are treated as though the input is a signed integer in **base**.

If `--quiet` is specified then parsing error messages will be suppressed.

When `--filter` is specified STDIN is filtered for inclusion addresses/masks given as arguments. Use `--filternum` to set the space delimited parameter that contains the input address.

Using `--abuseipdb [key]` can populate `%{abuseipdb_var}` values for `--format` strings, where `var` is the field name in a abuseipdb result structure.

# CSV

Network matches can be returned from a **CSV**.

    $ cat nets.csv
    network,range,owner
    rfc1918,192.168.0.0/16,bob
    rfc1918,172.16.0.0/12,cliff
    rfc1918,10.0.0.0/8,mr nobody
    $ ripcalc --csv nets.csv -i range --format '%{owner}\n' 192.168.0.0
    bob

Addresses can be read via file or from stdin (-):

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

# FORMAT

**%** denotes a format control character, followed by one of the following:

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
| %C          | In encapsulated context, used address count |
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

**%xa** gives the address in hex, or **%Sa** to return the binary address, split at the network boundary.

When using **CSV** fields can be matched by **name** when network matched:

    --format '%{name}'

# inside/outside

When `--inside` or `--outside` are given addresses that match `--file` are printed. If no matches are found `ripcalc` will exit non-zero.

# subnets

For large networks it can be useful to see the number of subnets, to see the number of /29 subnets within a /24 network, the command would look like this:

    ripcalc --networks 29 192.168.230.0/24
                IP is: 192.168.230.0/24
         Broadcast is: 192.168.230.255
           Network is: 192.168.230.0
            Subnet is: 255.255.255.0
          Wildcard is: 0.0.0.255
        Networks (29): 32

Or for a IPv6 /48 network that you want to subnet into /64, you can see there are 65536 subnets:

     ripcalc --networks 64 2001:db8:1::/48
                    IP is: 2001:db8:1::/48
             Expanded: 2001:0db8:0001:0000:0000:0000:0000:0000
           Network is: 2001:0db8:0001:0000:0000:0000:0000:0000
    Last host address: 2001:0db8:0001:ffff:ffff:ffff:ffff:ffff
            Subnet is: ffff:ffff:ffff:0000:0000:0000:0000:0000
        Networks (64): 65536

# encapsulating

Suppose a large flood of requests are from a network pattern, to preserve service you may want to block the whole network that encapsulates a list:

    please ip route add blackhole `ripcalc -e 192.168.56.10 192.168.57.1 192.168.44.47`

Networks can be grouped, in a scenario where you have a list of unwanted traffic, you can turn this into a list of small networks to block, supposing you don't want to block anything that covers more than a /19:

    cat bad_traffic | ripcalc --encapsulating --group 19 --format cidr

When using `group` the unique IP address count is available in the `%C` format string. This can give an overview of which networks have most IP sources:

    cat bad_traffic | ripcalc --encapsulating --group 19 --format '%C %a/%c\n' | sort -rn

# ipdb

CDB files can be used for address lookup. `--cdb` should hold a path to a valid CDB file, in which a CIDR network address should be used as a key, and NULL-separated key=value pairs should be the lookup data.

CDB files can be created with `--makecdb`, which reads comma separated input, the first value is a CIDR network, followed by `,` joined key=value pairs. See `cdb_maker.pl` for an example script which can be used to create such files.

The option `--makethymecdb` can be used to download an IP database from `https://thyme.apnic.net/`.

