# vim:set noet:
NAME := ripcalc
DOC := $(NAME)
VERSION := $(shell grep version Cargo.toml | sed -e 's/.* = "//g;s/"$$//g' )
MDDATE := $(shell find ripcalc.md -printf "%Td %TB %TY\n" )
RELEASE := ./target/release/ripcalc

all: build test bintest doc

build:
	cargo build --release

doc:
	( cat $(DOC).md | sed -e 's/^footer: \(\S\+\) \S\+$$/footer: \1 $(VERSION)/g' -e 's/^date:.*/date: $(MDDATE)/g' ) > $(DOC).md.tmp && mv $(DOC).md.tmp $(DOC).md
	cat $(DOC).md | sed -e 's,\([^ `-]\)--\([a-zA-Z]\),\1\\--\2,g' -e '/^|/s/\\n/\\\\n/g' -e '/^|/s/\\t/\\\\t/g' > $(DOC).man.md
	pandoc --standalone --ascii --to man $(DOC).man.md -o $(DOC).1
	rm $(DOC).man.md

test:
	cargo test

bintest:
	printf "127.0.0.1\n" | $(RELEASE) --available --list --format short -s - 127.0.0.1/30 | wc -l | grep -x 3
	printf "127.0.0.1\n" | $(RELEASE) --list --format short -s - 127.0.0.1/30 | wc -l | grep -x 5
	printf '10.0.0.0/30\n' | $(RELEASE) --list --format short -s - 127.0.0.1/30 | wc -l | grep -x 8
	printf '10.0.0.0/30\n127.0.0.1/30' | $(RELEASE) --list --format short -s - --outside 10.0.0.0/24 | wc -l | grep -x 4
	printf '10.0.0.0/28\n127.0.0.1/30' | $(RELEASE) --list --format short -s - --inside 10.0.0.0/24 | wc -l | grep -x 16
	printf '10.0.0.0/28\n127.0.0.1/30' | $(RELEASE) --list --format short --inside 10.0.0.0/24 | wc -l | grep -x 16
	printf '85.119.82.90\n' | $(RELEASE) -s - --inside 85.119.82.99/16 192.73.234.6/24 45.77.251.199/24 --format short | grep 85.119.82.90 | wc -l | grep -x 1
	printf '10.0.0.0/28\n127.0.0.1/30' | $(RELEASE) --list --format short -s - --inside 10.0.0.0/24 192.168.1.1/16 | wc -l | grep -x 16
	printf '127.0.0.1\n' | $(RELEASE) --format short -s - --inside 10.0.0.0/24 192.168.1.1/16 127.0.0.1/28 | wc -l | grep -x 1
	$(RELEASE) --format '%k.all.s5h.net' 127.0.0.2 # if there's no list/available option AND has an IP as an argument, it should not wait
	echo 192.168.1.2 | $(RELEASE) -l --inside 192.168.1.1/28 --format 'short' | wc -l | grep -x 1
	echo 192.168.0.0/16 | $(RELEASE) -l --inside 192.168.0.0/24 --format 'short' | wc -l | grep -x 0
	echo 192.168.0.0/28 | $(RELEASE) --list --noexpand --inside 192.168.0.0/24 --format 'short' | wc -l | grep -x 1
	echo 192.168.0.0/28 | $(RELEASE) --list -a --noexpand --inside 192.168.0.0/24 --format 'short' | wc -l | grep -x 1
	echo 338288524927261089655243473518709748348 | $(RELEASE) --base 10 -s - --format 'short' | grep -x fe80::10fe:91ff:fe64:b27c
	echo 3558236161 | $(RELEASE) --base 10 -s - --format 'short' | grep -x 212.22.96.1
	echo '185.27.20.54' | $(RELEASE) --outside 185.27.20.54/23 --format short | wc -l | grep -Fx 0
	printf '192.168.1.1\n192.168.2.1\n127.0.0.1\n10.10.10.10\n192.168.3.1\n' | $(RELEASE) --inside 192.168.1.0/24 192.168.2.0/24 --format short | wc -l | grep -Fx 2
	printf '192.168.1.1\n' | $(RELEASE) --format short --inside 80.87.128.0/20 185.27.20.0/22 216.116.64.0/20 67.214.98.0/24 2606:1F00::/32 2a04:1300::/29 | wc -l | grep -Fx 0
	printf '192.168.1.1\n' | $(RELEASE) --format short --outside 80.87.128.0/20 185.27.20.0/22 216.116.64.0/20 67.214.98.0/24 2606:1F00::/32 2a04:1300::/29 | wc -l | grep -Fx 1
	printf 'https://www.usenix.org.uk/content/\n' | $(RELEASE) --format short -s - | wc -l | grep -Fx 1
	printf '2001067c26600425001d0000000003d2' | $(RELEASE) --base 16 --format short -s - | wc -l | grep -Fx 1
	$(RELEASE) -e 10.0.0.0 10.10.0.0 --format cidr | grep 10.0.0.0/12

install: all
	command -v please && please install -m 0755 -s $(RELEASE) /usr/local/bin || sudo install -m 0755 -s $(RELEASE) /usr/local/bin 

