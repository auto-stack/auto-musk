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
            ['a2r_std::fs::read_to_string', sub {
                my ($a) = split_top_comma($_[0]);
                return "std::fs::read_to_string($a)";
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

# Plan 020 (feature_dev.at): a2r resolves Agent::run's `~Result<AgentResult,
# AgentError>` and injects `use crate::error::AgentError;` assuming the file
# lives in auto-ai-agent. In musk (where the module is transpiled) that path
# doesn't exist (E0432); the generated code never names AgentError explicitly
# (only extern agent_error_msg handles it), so the import is dropped.
$src =~ s/^use crate::error::AgentError;\s*\n//gm;

# Inject/fix extern_impl glob import so the .a2r.rs can call the glue-layer
# stubs (value_get_str, parse_json, specs_load, etc.) that live in
# extern_impl.rs. Use `super::` because every transpiled product lives in the
# auto_generated module alongside extern_impl (they are sibling modules).
# a2r emits `use super::extern_impl::*` already; if it emitted `crate::` (some
# files) rewrite it, otherwise inject it.
if ($src =~ /use crate::extern_impl/) {
    $src =~ s/use crate::extern_impl/use super::extern_impl/g;
} elsif ($src !~ /use super::extern_impl/) {
    $src = "use super::extern_impl::*;\n" . $src;
}

# Plan 384: strip the synthetic `fn main()` that a2r emits for top-level
# `let xxx_schema = \`...\`` constants. These schema strings are provided as
# `pub const` by extern_impl.rs (imported via the glob above), so the fn-main
# shadowing copies (which also have malformed format! braces) must be removed.
# Only matches a fn main whose body is exclusively schema `let` bindings.
$src =~ s/\nfn main\(\) \{\s*(?:let \w+_schema: String = format!\([^;]*;;\s*)+\}\s*\n/\n/g;

# Plan 384: fix const type annotations. a2r renders `const X str = "...";` as
# `const X: String = "...";` (or `: /* unknown */`), but a &'static str literal
# cannot be a `String` (not a const expression) and `/* unknown */` is illegal.
# Coerce both to `&str`.
$src =~ s/^(const \w+:) (String|\*\/ unknown \*\/) =/\1 \&str =/gm;

# Plan 384 stage-2: post-process fixes for a2r streaming/ownership bugs that
# are too invasive to fix in the transpiler right now.
# (1) async void fn (`-> ()`): a2r emits `return None;` for bare `return`, but
#     Rust `-> ()` fns need `return;`. a2r's fix_void handles bare `fn f()` but
#     misses some `async fn ... -> ()` cases (nested scope). Scope-aware rewrite:
#     only inside fns whose signature shows `-> ()`.
{
    my @lines = split /\n/, $src;
    my $in_void = 0; my $void_brace = 0; my $brace = 0;
    for my $i (0..$#lines) {
        my $t = $lines[$i]; $t =~ s/^\s+//; $t =~ s/\s+$//;
        if ($t =~ /^(pub )?(async )?fn / && ($t =~ /-> \(\)/ || $t =~ /->\(\)/ || $t !~ /->/)) {
            $in_void = 1; $void_brace = $brace;
        }
        for my $ch (split //, $t) { $brace++ if $ch eq '{'; $brace-- if $ch eq '}'; }
        if ($in_void && $brace <= $void_brace && $t =~ /}/) { $in_void = 0; }
        # match `return None;` (stmt) and `return None,` (match arm), inline.
        # Keep the original terminator (; for stmt, , for match arm).
        if ($in_void && $t =~ /return None([;,])/) { my $term = $1; $lines[$i] =~ s/return None[;,]/return$term/; }
    }
    $src = join("\n", @lines);
}
# (2) impl Stream with a &str param captures a lifetime not in bounds → add `+ '_`.
#     Skip conv_event_stream (its id is made owned in 2b below — no &str left).
$src =~ s/(fn (?!(?:conv_event_stream))\w+_stream\([^)]*&str[^)]*\) -> impl futures::Stream<Item = Result<Event, Infallible>>)( \{)/$1 + '_$2/g;
# (2b) conv_event_stream specifically: the stream outlives the caller's local
# `id`, so an `&str` capture dangles (E0597). Make `id` owned (String) and pass
# the owned value at the call site (move, not borrow).
$src =~ s/fn conv_event_stream\(rx: Value, id: &str\)/fn conv_event_stream(rx: Value, id: String)/g;
$src =~ s/conv_event_stream\(rx\.clone\(\), id\.as_str\(\)\)/conv_event_stream(rx, id)/g;
# (2c) inside conv_event_stream, conv_event_matches takes &str but id is now
#      owned String — borrow it at the call site.
$src =~ s/conv_event_matches\((&ev), id\)/conv_event_matches($1, \&id)/g;
# (3) ExitRouting::Loop tuple-ctor → struct-ctor (upstream uses named fields).
$src =~ s/ExitRouting::Loop\("([^"]*)",\s*(\d+)\)/ExitRouting::Loop { target_step_id: "$1".to_string(), max_iterations: $2 }/g;

# (3b) Plan 020 (feature_dev.at): AdvanceResult match patterns — a2r's type
# table (from auto-ai-agent .at sources) models tuple variants, but the runtime
# crate (rust-ref) declares struct variants → rewrite match patterns to the
# struct form (step_id/role_id/error/reason field names from rust-ref).
$src =~ s/AdvanceResult::ExecuteStep\((\w+), (\w+)\)/AdvanceResult::ExecuteStep { step_id: $1, role_id: $2 }/g;
$src =~ s/AdvanceResult::WaitForHuman\((\w+)\)/AdvanceResult::WaitForHuman { step_id: $1 }/g;
$src =~ s/AdvanceResult::Failed\((\w+)\)/AdvanceResult::Failed { error: $1 }/g;
$src =~ s/AdvanceResult::Paused\((\w+), (\w+)\)/AdvanceResult::Paused { step_id: $1, reason: $2 }/g;

# (4) OwnedRole needs `impl auto_ai_agent::Role` (delegates to inner Arc<dyn Role>).
# Auto's str→String is incompatible with Role's &str returns, so this is injected
# here rather than expressed in lib.at. Only inject if not already present.
if ($src =~ /struct OwnedRole/ && $src !~ /impl auto_ai_agent::Role for OwnedRole/) {
    my $role_impl = <<'RUST';
impl auto_ai_agent::Role for OwnedRole {
    fn name(&self) -> &str { self.inner.name() }
    fn system_prompt(&self) -> &str { self.inner.system_prompt() }
    fn model_tier(&self) -> ModelTier { self.inner.model_tier() }
    fn model(&self) -> &str { self.inner.model() }
    fn temperature(&self) -> f64 { self.inner.temperature() }
    fn max_turns(&self) -> usize { self.inner.max_turns() }
    fn allowed_tools(&self) -> Vec<String> { self.inner.allowed_tools() }
    fn memory_limit(&self) -> Option<usize> { self.inner.memory_limit() }
    fn allowed_tiers(&self) -> Vec<ModelTier> { self.inner.allowed_tiers() }
    fn token_budget(&self) -> Option<u64> { self.inner.token_budget() }
    fn skills(&self) -> Vec<String> { self.inner.skills() }
}
RUST
    $src .= $role_impl;
}

open my $out, '>', $file or die "cannot write $file: $!\n";
print $out $src;
close $out;
