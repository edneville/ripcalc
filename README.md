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

# integer inputs

To process addresses in signed integer form, use `--base -10` which will do the conversion.

To quickly format human-readable IP addresses to a database that stores addresses as integers, assuming addreses are in a table called IP4 and are signed integers:

```
printf '10.0.0.0/8\n192.168.0.0/16\n172.16.0.0/12\n127.0.0.0/8\n1.0.0.0/8\n' \
| ripcalc --format 'select ip from IP4 where active = 1 and (ip > %Ln and ip < %Lb);\n' \
| mysql -urip -pcalc ipdata \
| ripcalc -q --base 10 --format short
```

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

This might be of help to you, assuming you can select a chunk of bad traffic via a `grep`:

```
grep -h scams log/* | awk '{print $1}' | ripcalc --encapsulating --group 24 --format '%C %a/%c\n'
15 192.168.1.0/25
12 172.16.80.0/24
1 172.16.209.9/32
```

Grouping with a limit helps show spread, without expanding beyond sensible limits. Setting a very large `group` mask would of course return very large results, using a smaller `group` can limit this so the probability of including legitimate traffic is reduced dramatically.

IP address counts can be shown with `countseen` and `group`:

```
cat log | ripcalc --countseen --quiet --format '%C %a/%c\n'
```

# cdb

CDB databases are fast and if you have a large list of networks they offer a good way to lookup addresses without parsing CSV. CDB lookups can be performed using IP/cidr keys and the returned values should be stored as a NULL joined key=value string, e.g.:

```
192.168.1.0/24 name=lan\0owner=build\0service=web
```

The `--makethymecdb` argument will download ASN information from thyme.apnic.net and build that into a `cdb` file, or you can make your own. I find the data from thyme perfect for my needs, but this is the internet and geo-IP, please don't rely on geo-IP being accurate.

Obviously, if you have a fleet of computers using this data, it is best to download and build in one location then fan it out using `rsync` to your other computers to save resources and processing power.

Files can also be made using `--makecdb`, the stdin should be in the format of:

```
2600:3c0f:25::/48,ASNDESC=AKAMAI-LINODE-AP Akamai Connected Cloud,ASNCC=SG,ASN=2600:3c0f:25::/48,NET=63949
```

See cdb_maker.pl for a helper script which builds this. An example for making these can be seen in the Makefile in the ipdb target, it download ASN data and builds that using cdb_maker.pl, and this can be used like this:

```
ripcalc --cdb ipdb.cdb --format '%{ASNDESC} %{ASNCC} %a/%c\n' 1.1.1.1
CLOUDFLARENET US 1.1.1.1/24
```

# filter

Input can be filtered using `--filter` which can be helpful as an accompaniment for `grep`-like workflows.

```
cat /var/log/apache/access.log | ripcalc --filter 1.2.3.0/24
```

Or inverted with `--outside`.

```
cat /var/log/apache/access.log | ripcalc --filter 1.1.1.0/24 --outside
```

`--filter` will use the first word in the inputline and try to parse that for an IP address, if the IP address is not the first word in the input, then `--filternum` can be used to indicate which word number holds the IP.

If you'd like to see the country code that the requests in your log are from:

```
cat /var/log/apache/access.log | ripcalc --filter --outside --cdb ipdb.cdb --format '%{ASNCC} %{line}\n'
```

If you want to print certain IP ranges, perhaps for an email report or similar, the following could work:

```
#!/bin/sh

A=`mktemp`
cat /var/log/apache2/access.log | ripcalc --filter --inside 192.168.0.0/16 >"$A"

if [ -s "$A" ]; then
        (
        printf "From: me@example.com\nTo: me@example.com\nSubject: 192.168.0.0/16 access\n\n"
        cat "$A"
        ) | /usr/sbin/sendmail -fme@example.com me@example.com
fi

rm "$A"

```

# abuseipdb

If `--abuseipdb [key]` is used then a successful lookup will populate `%{abuseipdb_var}` (where `var` is the lookup result data field) for use in a `--format` string.

Replace `[key]` with your abuseipdb API key.

If only the country code or ISP is needed, then using a locally generated `cdb` is more efficient, see the CDB section above as that can be much faster if you don't require an external abuse score.

# maxmind geolite

MaxMind's geolite database files can be read using `--mmasn`, `--mmcountry` and `--mmcity` options. The text language can be defined with `--mmlang`.

In normal context the following are returned:

mmcity: `%{mmcity_city}`, `%{mmcity_continent}`, `%{mmcity_country}`, `%{mmcity_code}`, `%{mmcity_latitude}`, `%{mmcity_longitude}`

mmcountry: `%{mmcountry_continent}`, `%{mmcountry_country}`, `%{mmcountry_registered_country}`

mmasn: `%{mmasn_autonomous_system_number}`, `%{mmasn_autonomous_system_organization}`

Less information is returned in `--top` context:

```
ripcalc --top --allowemptyrow \
    --mmcity GeoLite2-City.mmdb \
    --mmcountry GeoLite2-Country.mmdb \
    --mmasn GeoLite2-ASN.mmdb 
```

Without needing to add a `--format` string, in `--top` context, this will display `%{mmcity_code}` and `%{mmcity_country}` for `--mmcity`, `%{mmasn_autonomous_system_organization}` for `--mmasn`, and `%{mmcountry_registered_country}` for `--mmcountry`.

# top

Basic report of IP frequency, as read from `stdin`. Can be used with `--group`, `--group[46]`, `--filter`, `--filterany`, `--filternum` or `--inside` to show which networks appear most frequently.

```
ripcalc --allowemptyrow --top --inside 10.0.0.0/8 --group 24 --delay 1

                                                   10.12.0.0/14 4800 ##
                                                   10.16.0.0/14 4075 ##
                                                    10.8.0.0/14 3740 #
                                                    10.4.0.0/14 3200 #
                                                    10.0.0.0/14 2400 #
                                                   10.20.0.0/14  800 

```

A cdb IP database can be used in conjunction, which defaults to `%{ASNCC} %{ASNDESC}`:

```
ripcalc --top --inside 10.0.0.0/8 --group 14 --delay 1 --allowemptyrow --cdb ipdb.cdb
XX PRIVATE                                          10.8.0.0/14 3200 ##
XX PRIVATE                                          10.4.0.0/14 3200 ##
XX PRIVATE                                         10.12.0.0/14 3200 ##
XX PRIVATE                                         10.16.0.0/14 3200 ##
XX PRIVATE                                          10.0.0.0/14 2400 #
XX PRIVATE                                         10.20.0.0/14  800 
```

If you want to show the abuse score from AbuseIPDB at the same time you can use a format string or allow the default to be used which shows the abuse score.

`--iterations [num]` can set the number of display iterations. If you wish to run a number of displays for an email report, use with `--noclear`:

```
( printf 'To: you\nFrom: me\nSubject: IP report\n\n';
tail -Fq /var/log/apache2/*log
| ripcalc --top --cdb ipdb.cdb --group4 24 --group6 64 --allowemptyrow --iterations 3 --noclear )
| /usr/sbin/sendmail -fme@example you@example
```

All input can be read with `--consume` so a report can be produced.

# help

```

ripcalc version 0.3.0

Options:
    -4, --ipv4          treat inputs as ipv4 address
    -6, --ipv6          treat inputs as ipv6 address
    -a, --available     display unused addresses
        --abuseipdb KEY abuseipdb API
        --allowemptyrow 
                        when no matching cdb/csv network, use empty fields
    -b, --base INTEGER  ipv4 base format, default to oct
        --cdb PATH      cdb reference file
        --countseen     count times an ip is seen
    -c, --csv PATH      csv reference file
        --consume       read stdin until empty
    -d, --divide CIDR   divide network into chunks
        --noexpand      do not expand networks in list
    -e, --encapsulating 
                        display encapsulating network from arguments or lookup
                        list
    -f, --format STRING format output
                        'cidr' expands to %a/%c\n
                        'short' expands to %a\n
                        See manual for more options
        --filter        print STDIN line if field parses as an IP and matches,
                        defaults to 1, use filternum to adjust
        --filterany     print STDIN line if any IP on STDIN matches
        --filternum NUMBER
                        change position NUMBER for IP filter, enables --filter
        --group CIDR    maximum network group size for encapsulation
        --group4 CIDR   v4 specific group, overrides group in top mode
        --group6 CIDR   v6 specific group, overrides group in top mode
    -h, --help          display help
    -i, --field FIELD   csv/db/json field
    -l, --list          list all addresses in network
        --outside       display when extremities are outside network
        --inside        display when extremities are inside network
        --makecdb PATH  build a cdb file from STDIN
        --makethymecdb PATH
                        download and build a cdb file, defaults to
                        thyme.apnic.net data
        --data-raw-table URL or PATH
                        URL/path of raw data
        --ipv6-raw-table URL or PATH
                        URL/path of ipv6 data
        --data-used-autnums URL or PATH
                        URL/path of ASN data
        --top           show IP frequency like top
        --delay SECONDS top update delay in seconds
        --iterations NUMBER
                        top iterations, use --consume to produce whole report
                        of stdin
        --noclear       don't print clear screen codes
        --mmlang LANG   the lang to use in results, defaults to en
        --mmasn PATH    path to maxmind geoip asn file
        --mmcity PATH   path to maxmind geoip city file
        --mmcountry PATH
                        path to maxmind geoip country file
    -m, --mask CIDR     cidr mask
    -n, --networks CIDR instead of hosts, display number of subnets of this
                        size
        --json PATH     json prefix file
    -q, --quiet         don't report errors
    -r, --reverse       (none, inputs, sources or both) v4 octets, v6 hex
    -s, --file PATH     lookup addresses from, - for stdin
    -v, --version       print version
```

