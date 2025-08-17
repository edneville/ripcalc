#!/usr/bin/perl

# script to process files from thyme.apnic.net
#
# CC from the ASN description can then be used like this
# ripcalc --cdb ipdb.cdb --format '%a/%c %{ASNCC}\n' 2001:BA8:1F1:F1CB:0:0:0:3
# 2001:ba8:1f1:f1cb::3/32 GB
# 
# this isn't limited to getting CC code, you can store anything you like
# in CDB files, web crawlers/scrapers etc, customer networks etc

use strict;
use warnings;
use Data::Dumper;

my %db;
my %net;

open( my $asn,  "<", $ARGV[0] );
while (my $line = <$asn>) {
    chomp($line);
    if ($line =~ /^\s*([0-9]+)\s+([0-9]+)\s+([0-9]+)\s+(.+)\s*$/) {
        $db{$1} = $4;
    }
}
close($asn);

open( my $prefix,  "<", $ARGV[1] );
while (my $line = <$prefix>) {
    chomp($line);
    if ($line =~ /^\s*([0-9]+)\s+(\S+)$/) {
        if (exists($db{$1})) {
            my ($desc,$cc) = split(/,\s*/, $db{$1}, 2);
            if (defined($desc) && defined($cc)) {
                $net{$2} = "ASNDESC=$desc,ASNCC=$cc,ASN=$1,NET=$2";
            }
        }
    }
}
close($asn);

open( my $ipv6,  "<", $ARGV[2] );
while (my $line = <$ipv6>) {
    chomp($line);
    if ($line =~ /^\s*(\S+)\s+(\S+)$/) {
        if (exists($db{$2})) {
            my ($desc,$cc) = split(/,\s*/, $db{$2}, 2);
            if (defined($desc) && defined($cc)) {
                $net{$1} = "ASNDESC=$desc,ASNCC=$cc,ASN=$1,NET=$2";
            }
        }
    }
}
close($asn);

for my $k (keys %net) {
    print "$k,$net{$k}\n";
}

