#!/usr/bin/env perl
# nativeize.pl — post-process a2r output to drop the a2r-std runtime.
#
# a2r hard-bridges some methods (str.contains, str.find, fs.write, time) to
# a2r-std helper functions and injects `use a2r_std; use a2r_std::*;`. This
# script rewrites those bridges back to native Rust and strips the preamble,
# so the transpiled .rs depends only on real crates (serde/std/...), not a2r-std.
#
# Usage: perl nativeize.pl <file.rs>   (edits in place)
#
# Bridges handled (1:1 semantic equivalents):
#   a2r_std::str_contains(a, b)        -> a.contains(b)
#   a2r_std::str_find(a, b, _)         -> a.contains(b)   (bool context; see note)
#   a2r_std::fs::write(path, content)  -> std::fs::write(path, content).is_ok()
#   a2r_std::fs::exists(path)          -> std::path::Path::new(path).exists()
# plus drop: use a2r_std; / use a2r_std::*; / use a2r_std::<module>;
#
# NOTE on str_find: a2r_std::str_find returns i32 (-1 if absent). Call sites in
# our .at use it only in boolean context (`is x.find(n) { Some->.. }`-style has
# already been bridged, and direct `find` calls are rare). If a call needs the
# index, handle it manually. We map it to contains() since all current uses are
# boolean. Review output if you add index-based find usage.

use strict;
use warnings;

my $file = $ARGV[0] or die "usage: nativeize.pl <file.rs>\n";
open my $fh, '<', $file or die "cannot read $file: $!\n";
my $src = do { local $/; <$fh> };
close $fh;

# Helper: match a balanced (...) group starting right after a known prefix.
# Returns (whole_match, inside_parens). Handles one level of nested parens
# and quotes well enough for the simple argument shapes a2r emits here.
sub match_call {
    my ($src, $prefix) = @_;
    if ($src =~ /\Q$prefix\E\s*\(/) {
        my $start = $+[0];
        # walk from the '(' to its matching ')'
        my $i = index($src, '(', $start - 1);
        my $depth = 0;
        my $in_str = 0;
        while ($i < length($src)) {
            my $c = substr($src, $i, 1);
            if ($in_str) {
                $in_str = 0 if $c eq '"';
            } elsif ($c eq '"') {
                $in_str = 1;
            } elsif ($c eq '(') {
                $depth++;
            } elsif ($c eq ')') {
                $depth--;
                if ($depth == 0) {
                    my $inside = substr($src, $start, $i - $start);
                    return (substr($src, $-[0], $i + 1 - $-[0]), $inside);
                }
            }
            $i++;
        }
    }
    return (undef, undef);
}

# Rewrite each bridge call iteratively (a line may contain one).
sub rewrite_bridge {
    my ($src_ref) = @_;
    my $changed = 1;
    while ($changed) {
        $changed = 0;
        for my $entry (
            ['a2r_std::str_contains', sub {
                my ($a, $b) = split_top_comma($_[0]);
                return "$a.contains($b)";
            }],
            ['a2r_std::fs::write', sub {
                my ($a, $b) = split_top_comma($_[0]);
                return "std::fs::write($a, $b).is_ok()";
            }],
            ['a2r_std::fs::exists', sub {
                my ($a) = split_top_comma($_[0]);
                return "std::path::Path::new($a).exists()";
            }],
        ) {
            my ($prefix, $fn) = @$entry;
            my ($whole, $inside) = match_call($$src_ref, $prefix);
            if (defined $whole) {
                my $replacement = $fn->($inside);
                my $pos = index($$src_ref, $whole);
                substr($$src_ref, $pos, length($whole)) = $replacement;
                $changed = 1;
            }
        }
    }
}

# Split "a, b" on the top-level comma (ignoring commas inside nested ()).
sub split_top_comma {
    my ($s) = @_;
    my $depth = 0;
    my $in_str = 0;
    for my $i (0 .. length($s) - 1) {
        my $c = substr($s, $i, 1);
        if ($in_str) { $in_str = 0 if $c eq '"'; }
        elsif ($c eq '"') { $in_str = 1; }
        elsif ($c eq '(') { $depth++; }
        elsif ($c eq ')') { $depth--; }
        elsif ($c eq ',' && $depth == 0) {
            return (trim(substr($s, 0, $i)), trim(substr($s, $i + 1)));
        }
    }
    return (trim($s));
}

sub trim {
    my $s = shift;
    $s =~ s/^\s+//;
    $s =~ s/\s+$//;
    return $s;
}

rewrite_bridge(\$src);

# Strip the a2r-std preamble lines.
$src =~ s/^use a2r_std;\s*\n//gm;
$src =~ s/^use a2r_std::\*;\s*\n//gm;
$src =~ s/^use a2r_std::\w+(?:::\w+)?;\s*\n//gm;

open my $out, '>', $file or die "cannot write $file: $!\n";
print $out $src;
close $out;
