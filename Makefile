VERSION := $(shell grep version Cargo.toml | sed -e 's/.* = "//g;s/"$$//g' )
MDDATE := $(shell find ripcalc.md -printf "%Td %TB %TY\n" )

all:

build:
	cargo build --release

doc:
	( cat ripcalc.md | sed -e 's/^footer: \(\S\+\) \S\+$$/footer: \1 $(VERSION)/g' -e 's/^date:.*/date: $(MDDATE)/g' ) > ripcalc.md.tmp && mv ripcalc.md.tmp ripcalc.md
	cat ripcalc.md | sed -e '/^|/s/\\n/\\\\n/g' -e '/^|/s/\\t/\\\\t/g' > ripcalc.man.md
	pandoc --standalone --ascii --to man ripcalc.man.md -o ripcalc.1

test:
	cargo test

install: test build
	please install -m 0755 -s target/release/ripcalc /usr/local/bin

