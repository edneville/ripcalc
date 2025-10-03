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

my %db;
my %net;
my %asn;

sub process_raw {
    my $fh = shift;
    my $db = shift;
    my $asn = shift;

    while (my $line = <$fh>) {
        chomp $line;
        if ($line =~ m,^\s*([\d.]+\/\d+)\s+(\d+)\s*$,) {
            $db->{$1} = $2;
            push(@{$asn->{$2}}, $1);
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
        if ($line =~ /^\s*(\d+)\s+(.*)$/) {
            my ($desc,$cc) = split(/,\s*([^,]+)$/, $2);
            if (defined $desc && defined $cc) {
                $net->{$1} = "ASNDESC=$desc,ASNCC=$cc,ASN=$1,NET=$2";
            }
        }
    }

    return;
}

sub process_ipv6 {
    my $fh = shift;
    my $db = shift;
    my $asn = shift;

    while (my $line = <$fh>) {
        chomp $line;
        if ($line =~ /^\s*(\S+)\s+(\S+)$/) {
            $db->{$1} = $2;
            push(@{$asn->{$2}}, $1);
        }
    }

    return;
}

sub main {
    if (scalar(@ARGV) < 3) {
        print (STDERR "Usage: $0 data-raw data-asn data-ipv6\n");
        exit 1;
    }

    open my $fh,  '<', $ARGV[0] || die "cannot open ${ARGV[0]}";
    process_raw($fh, \%db, \%asn);
    close $fh;

    open my $add_fh,  '<', $ARGV[1] || die "cannot open ${ARGV[1]}";
    process_add($add_fh, \%db, \%net);
    close $add_fh;

    open my $ipv6_fh,  '<', $ARGV[2] || die "cannot open ${ARGV[2]}";
    process_ipv6($ipv6_fh, \%db, \%asn);
    close $ipv6_fh;

    for my $k (keys %db) {
        if ($net{$db{$k}}) {
            print ($k, ",", $net{$db{$k}}, "\n");
        }
    }

    for my $k (keys %asn) {
        my @p = @{$asn{$k}};
        print ($k, ",", join(",", @p), "\n");
    }

    return;
}

main;
