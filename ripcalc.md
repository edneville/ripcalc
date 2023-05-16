---
title: ripcalc
section: 1
header: User Manual
footer: ripcalc 0.1.7
author: Ed Neville (ed-ripcalc@s5h.net)
date: 16 May 2023
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

**ripcalc -s/--file [-] 127.0.0.1**

**ripcalc -e/--encapsulating -s**

**ripcalc -h/--help**

# DESCRIPTION

**ripcalc** can read IPv4/IPv6 addresses from command line or standard input and output different formats or associated networks from **CSV**.

**ripcalc** can format network addresses, find matches in **CSV** or process a list.

# CSV

Network matches can be returned from a **CSV**.

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
| %x          | Hex address string |

Other format characters:

| placeholder | effect |
|-------------|--------|
| %c          | CIDR mask |
| %t          | Network size |
| %r          | Network reservation information (if available) |
| %d          | Matching device interface by IP |
| %m          | Matching media link interface by network |
| %k          | RBL-style format |
| %%          | % |
| \n          | Line break |
| \t          | Tab character |

**%xa** gives the address in hex, or **%Sa** to return the binary address, split at the network boundary.

When using **CSV** fields can be matched by **name** when network matched:

```
--format '%{name}'
```

When `-a` is used, addresses read from `-s` will not be shown when listing `-l` a network, showing only available addresses.

