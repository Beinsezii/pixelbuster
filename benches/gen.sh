#!/usr/bin/env bash

# TODO: Figure out how to NOT run the benches 300 times each.
# 12 megapixels
# COUNT=$((4000 * 3000 * 4))
COUNT=$((800 * 600 * 4))
DIR="$( cd -- "$( dirname -- "${BASH_SOURCE[0]:-$0}"; )" &> /dev/null && pwd 2> /dev/null; )";

set -o noglob

# argv1: name
# argv2: ops
gen() {
    echo -e '#![feature(test)]

extern crate test;

use test::Bencher;

use pixelbuster::pbcore::{parse_ops, process, Space};

use fastrand;

const COUNT: usize = '$COUNT';
const OPS: &str = "'$2'";

fn gen_px() -> Vec<f32> {
    (0..COUNT).map(|_| fastrand::f32()).collect()
}

#[bench]
fn '$1'_parse(b: &mut Bencher) {
    b.iter(|| parse_ops(OPS, Space::SRGB));
}

#[bench]
fn '$1'_process(b: &mut Bencher) {
    let mut pixels = gen_px();
    let ops = parse_ops(OPS, Space::SRGB);
    b.iter(|| process(&ops.0, &mut pixels, 0, None));
}
' > "$DIR/${1}.rs"
}



gen "no_op" ""

gen "minimal" "r = r"

CODE=""
for _ in {1..256}; do CODE="${CODE}r = r\\\n"; done
gen "maximal" "$CODE"

gen "filmic_chroma" "
lch \n
v1 = 0.5 \n
v2 = 0.0 \n
v2 + 1 \n
v3 = 100 \n
v3 / v1 \n
v4 = l \n
v4 / v3 \n
v2 - v4 \n
c * v2 \n
"

gen "sweep" "
v1 = r \n
r + r \n
r - r \n
r * r \n
r / r \n
r % r \n
r pow r \n
r abs r \n
r acos r \n
r acosh r \n
r asin r \n
r asinh r \n
r atan r \n
r atan2 r \n
r atanh r \n
r cbrt r \n
r ceil r \n
r cos r \n
r cosh r \n
r floor r \n
r log r \n
r max r \n
r min r \n
r round r \n
r sin r \n
r sinh r \n
r sqrt r \n
r tan r \n
r tanh r \n
r = v1 \n
"

gen "space_minimal" "LCH"

gen "space_sweep" "
SRGB \n
LRGB \n
XYZ \n
LAB \n
LCH \n
LAB \n
XYZ \n
LRGB \n
SRGB \n
"
gen "space_jumps" "
SRGB \n
LCH \n
SRGB \n
LCH \n
SRGB \n
LCH \n
SRGB \n
LCH \n
SRGB \n
"
