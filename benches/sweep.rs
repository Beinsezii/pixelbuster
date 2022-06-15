#![feature(test)]

extern crate test;

use test::Bencher;

use pixelbuster::pbcore::{parse_ops, process, Space};

use fastrand;

const COUNT: usize = 1920000;
const OPS: &str = " v = r 
 r + r 
 r - r 
 r * r 
 r / r 
 r % r 
 r pow r 
 r abs r 
 r acos r 
 r acosh r 
 r asin r 
 r asinh r 
 r atan r 
 r atan2 r 
 r atanh r 
 r cbrt r 
 r ceil r 
 r cos r 
 r cosh r 
 r floor r 
 r log r 
 r max r 
 r min r 
 r round r 
 r sin r 
 r sinh r 
 r sqrt r 
 r tan r 
 r tanh r 
 r = v 
 ";

fn gen_px() -> Vec<f32> {
    (0..COUNT).map(|_| fastrand::f32()).collect()
}

#[bench]
fn sweep_parse(b: &mut Bencher) {
    b.iter(|| parse_ops(OPS, Space::SRGB));
}

#[bench]
fn sweep_process(b: &mut Bencher) {
    let mut pixels = gen_px();
    let ops = parse_ops(OPS, Space::SRGB);
    b.iter(|| process(&ops.0, &mut pixels, 0, None));
}

