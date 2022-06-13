#![feature(test)]

extern crate test;

use test::Bencher;

use pixelbuster::pbcore::{parse_ops, process, Space};

const COUNT: usize = 1920000;
const OPS: &str = "r = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\nr = r\n";

fn gen_px() -> Vec<f32> {
    (0..COUNT).map(|_| rand::random::<f32>()).collect()
}

#[bench]
fn maximal_parse(b: &mut Bencher) {
    b.iter(|| parse_ops(OPS, Space::SRGB));
}

#[bench]
fn maximal_process(b: &mut Bencher) {
    let mut pixels = gen_px();
    let ops = parse_ops(OPS, Space::SRGB);
    b.iter(|| process(&ops.0, &mut pixels, None));
}

