#![feature(test)]

extern crate test;

use test::Bencher;

use pixelbuster::pbcore::{parse_ops, process, Space};

const COUNT: usize = 1920000;
const OPS: &str = " lch 
 v1 = 0.5 
 v2 = 0.0 
 v2 + 1 
 v3 = 100 
 v3 / v1 
 v4 = l 
 v4 / v3 
 v2 - v4 
 c * v2 
 ";

fn gen_px() -> Vec<f32> {
    (0..COUNT).map(|_| rand::random::<f32>()).collect()
}

#[bench]
fn filmic_chroma_parse(b: &mut Bencher) {
    b.iter(|| parse_ops(OPS, Space::SRGB));
}

#[bench]
fn filmic_chroma_process(b: &mut Bencher) {
    let mut pixels = gen_px();
    let ops = parse_ops(OPS, Space::SRGB);
    b.iter(|| process(&ops.0, &mut pixels, None));
}

