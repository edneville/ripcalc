VERSION := $(shell grep version Cargo.toml | sed -e 's/.* = "//g;s/"$$//g' )
MDDATE := $(shell find ripcalc.md -printf "%Td %TB %TY\n" )

all:

build:
	cargo build --release

doc:
	( cat ripcalc.md | sed -e 's/^footer: \(\S\+\) \S\+$$/footer: \1 $(VERSION)/g' -e 's/^date:.*/date: $(MDDATE)/g' ) > ripcalc.md.tmp && mv ripcalc.md.tmp ripcalc.md
	cat ripcalc.md | sed -e 's,\([^ `-]\)--\([a-zA-Z]\),\1\\--\2,g' -e '/^|/s/\\n/\\\\n/g' -e '/^|/s/\\t/\\\\t/g' > ripcalc.man.md
	pandoc --standalone --ascii --to man ripcalc.man.md -o ripcalc.1
	rm ripcalc.man.md

test:
	cargo test

bintest:
	printf "127.0.0.1\n" | ./target/release/ripcalc --available --list --format short -s - 127.0.0.1/30 | wc -l | grep -x 3
	printf "127.0.0.1\n" | ./target/release/ripcalc --list --format short -s - 127.0.0.1/30 | wc -l | grep -x 260
	printf '10.0.0.0/30\n' |./target/release/ripcalc --list --format short -s - 127.0.0.1/30 | wc -l | grep -x 8
	printf '10.0.0.0/30\n127.0.0.1/30' |./target/release/ripcalc --list --format short -s - --outside 10.0.0.0/24 | wc -l | grep -x 4
	printf '10.0.0.0/28\n127.0.0.1/30' |./target/release/ripcalc --list --format short -s - --inside 10.0.0.0/24 | wc -l | grep -x 16
	printf '10.0.0.0/28\n127.0.0.1/30' |./target/release/ripcalc --list --format short --inside 10.0.0.0/24 | wc -l | grep -x 16

install: test build bintest
	command -v please && please install -m 0755 -s target/release/ripcalc /usr/local/bin || sudo install -m 0755 -s target/release/ripcalc /usr/local/bin 

