#![feature(test)]

extern crate test;

use test::Bencher;

use pixelbuster::pb_core::{parse_ops, process_multi, process_segment, Space};

const COUNT: usize = 1920000;
const OPS: &str = "LCH";

fn gen_px() -> Vec<f32> {
    (0..COUNT).map(|_| rand::random::<f32>()).collect()
}

#[bench]
fn space_minimal_parse(b: &mut Bencher) {
    b.iter(|| parse_ops(OPS, Space::SRGB));
}

#[bench]
fn space_minimal_process(b: &mut Bencher) {
    let mut pixels = gen_px();
    let ops = parse_ops(OPS, Space::SRGB);
    b.iter(|| process_multi(&ops, &mut pixels, None));
}
#[bench]
fn space_minimal_single(b: &mut Bencher) {
    let mut pixels = gen_px();
    let ops = parse_ops(OPS, Space::SRGB);
    b.iter(|| process_segment(&ops, &mut pixels, None));
}

