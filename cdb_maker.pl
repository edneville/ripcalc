#!/usr/bin/perl

# script to process files from thyme.apnic.net
#
# CC from the ASN description can then be used like this
# ripcalc --cdb ipdb.cdb --format '%a/%c %{ASNCC}\n' 2001:BA8:1F1:F1CB:0:0:0:3
# 2001:ba8:1f1:f1cb::3/32 GB
# 
# this isn't limited to getting CC code, you can store anything you like
# in CDB files, web crawlers/scrapers etc, customer networks etc
#
# normally
#
# perl cdb_maker.pl data-AS20net data-add ipv6-raw-table | ripcalc --makecdb ipdb.cdb

use strict;
use warnings;
use Data::Dumper;

my %db;
my %net;

sub process_as20 {
    my $fh = shift;
    my $db = shift;

    while (my $line = <$fh>) {
        chomp $line;
        if ($line =~ /^\s*(\d+)\s+(\d+)\s+(\d+)\s+(.+)\s*$/) {
            $db->{$1} = $4;
        }
    }

    return;
}

sub process_add {
    my $fh = shift;
    my $db = shift;
    my $net = shift;

    while (my $line = <$fh>) {
        chomp $line;
        if ($line =~ /^\s*(\d)\s+(\S+)$/) {
            if (defined $db->{$1}) {
                my ($desc,$cc) = split /,\s*/, $db->{$1}, 2;
                if (defined $desc && defined $cc) {
                    $net->{$2} = "ASNDESC=$desc,ASNCC=$cc,ASN=$1,NET=$2";
                }
            }
        }
    }

    return;
}

sub process_ipv6 {
    my $fh = shift;
    my $db = shift;
    my $net = shift;

    while (my $line = <$fh>) {
        chomp $line;
        if ($line =~ /^\s*(\S+)\s+(\S+)$/) {
            if (defined $db->{$2}) {
                my ($desc,$cc) = split /,\s*/, $db->{$2}, 2;
                if (defined $desc && defined $cc) {
                    $net->{$1} = "ASNDESC=$desc,ASNCC=$cc,ASN=$1,NET=$2";
                }
            }
        }
    }

    return;
}

open my $fh,  '<', $ARGV[0] || die "cannot open ${ARGV[0]}";
process_as20($fh, \%db);
close $fh;

open my $add_fh,  '<', $ARGV[1] || die "cannot open ${ARGV[1]}";
process_add($add_fh, \%db, \%net);
close $add_fh;

open my $ipv6_fh,  '<', $ARGV[2] || die "cannot open ${ARGV[2]}";
process_ipv6($ipv6_fh, \%db, \%net);
close $ipv6_fh;

for my $k (keys %net) {
    print "$k,$net{$k}\n";
}

