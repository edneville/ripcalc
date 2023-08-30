# vim:set noet:
NAME := ripcalc
DOC := $(NAME)
VERSION := $(shell grep version Cargo.toml | sed -e 's/.* = "//g;s/"$$//g' )
MDDATE := $(shell find ripcalc.md -printf "%Td %TB %TY\n" )

all:

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
	printf "127.0.0.1\n" | ./target/release/ripcalc --available --list --format short -s - 127.0.0.1/30 | wc -l | grep -x 3
	printf "127.0.0.1\n" | ./target/release/ripcalc --list --format short -s - 127.0.0.1/30 | wc -l | grep -x 5
	printf '10.0.0.0/30\n' | ./target/release/ripcalc --list --format short -s - 127.0.0.1/30 | wc -l | grep -x 8
	printf '10.0.0.0/30\n127.0.0.1/30' | ./target/release/ripcalc --list --format short -s - --outside 10.0.0.0/24 | wc -l | grep -x 4
	printf '10.0.0.0/28\n127.0.0.1/30' | ./target/release/ripcalc --list --format short -s - --inside 10.0.0.0/24 | wc -l | grep -x 16
	printf '10.0.0.0/28\n127.0.0.1/30' | ./target/release/ripcalc --list --format short --inside 10.0.0.0/24 | wc -l | grep -x 16
	printf '85.119.82.90\n' | ./target/release/ripcalc -s - --inside 85.119.82.99/16 192.73.234.6/24 45.77.251.199/24 --format short | grep 85.119.82.90 | wc -l | grep -x 1
	printf '10.0.0.0/28\n127.0.0.1/30' | ./target/release/ripcalc --list --format short -s - --inside 10.0.0.0/24 192.168.1.1/16 | wc -l | grep -x 16
	printf '127.0.0.1\n' | ./target/release/ripcalc --format short -s - --inside 10.0.0.0/24 192.168.1.1/16 127.0.0.1/28 | wc -l | grep -x 1
	./target/release/ripcalc --format '%k.all.s5h.net' 127.0.0.2 # if there's no list/available option AND has an IP as an argument, it should not wait
	echo 192.168.1.2 | ./target/release/ripcalc -l --inside 192.168.1.1/28 --format 'short' | wc -l | grep -x 1
	echo 192.168.0.0/16 | ./target/release/ripcalc -l --inside 192.168.0.0/24 --format 'short' | wc -l | grep -x 0
	echo 192.168.0.0/28 | ./target/release/ripcalc --list --noexpand --inside 192.168.0.0/24 --format 'short' | wc -l | grep -x 1
	echo 192.168.0.0/28 | ./target/release/ripcalc --list -a --noexpand --inside 192.168.0.0/24 --format 'short' | wc -l | grep -x 1
	echo 338288524927261089655243473518709748348 | ./target/release/ripcalc --base 10 -s - --format 'short' | grep -x fe80::10fe:91ff:fe64:b27c
	echo 3558236161 | ./target/release/ripcalc --base 10 -s - --format 'short' | grep -x 212.22.96.1
	echo '185.27.20.54' | ./target/release/ripcalc --outside 185.27.20.54/23 --format short | wc -l | grep -Fx 0

install: test build bintest
	command -v please && please install -m 0755 -s target/release/ripcalc /usr/local/bin || sudo install -m 0755 -s target/release/ripcalc /usr/local/bin 

