use criterion::{black_box, criterion_group, criterion_main, Criterion};
use pixelbuster::pbcore::{parse_ops, process, Space};
use fastrand;

const NO_OP: &str = "";
const MINIMAL: &str = "r = r";
const MAXIMAL: &str = "
r = r
r = r
r = r
r = r
r = r
r = r
r = r
r = r
r = r
r = r
r = r
r = r
r = r
r = r
r = r
r = r
r = r
r = r
r = r
r = r
r = r
r = r
r = r
r = r
r = r
r = r
r = r
r = r
r = r
r = r
r = r
r = r
r = r
r = r
r = r
r = r
r = r
r = r
r = r
r = r
r = r
r = r
r = r
r = r
r = r
r = r
r = r
r = r
r = r
r = r
r = r
r = r
r = r
r = r
r = r
r = r
r = r
r = r
r = r
r = r
r = r
r = r
r = r
r = r
";

const SWEEP: &str = "
v1 = r
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
r = v1
";

const SPACE_MINIMAL: &str = "OKLCH";
const SPACE_STEPS: &str = "SRGB; LRGB; XYZ; OKLAB; OKLCH";
const SPACE_MAXIMAL: &str = "HSV; LCH; OKLAB; JZCZHZ; OKLAB; LCH; HSV; SRGB";

const FILMIC_CHROMA: &str = "
lch 
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

macro_rules! bench_op {
    ($cr: expr, $id: literal, $op: expr, $image: expr) => {
        $cr.bench_function(concat!($id, "_parse"), |b| {
            b.iter(|| black_box(parse_ops($op, Space::SRGB)))
        });
        let ops = parse_ops($op, Space::SRGB).0;
        $cr.bench_function(concat!($id, "_process"), |b| {
            b.iter(|| {
                let mut image = $image.clone();
                black_box(process(&ops, &mut image, 0, None))
            })
        });
    };
}

fn ops_main(c: &mut Criterion) {
    let image: Vec<f32> = (0..(1024 * 1024 * 4)).map(|_| fastrand::f32()).collect();

    bench_op!(c, "no_op", NO_OP, image);
    bench_op!(c, "minimal", MINIMAL, image);
    bench_op!(c, "maximal", MAXIMAL, image);
    bench_op!(c, "sweep", SWEEP, image);
    bench_op!(c, "space_minimal", SPACE_MINIMAL, image);
    bench_op!(c, "space_steps", SPACE_STEPS, image);
    bench_op!(c, "space_maximal", SPACE_MAXIMAL, image);
    bench_op!(c, "filmic_chroma", FILMIC_CHROMA, image);
}

criterion_group!(ops, ops_main);
criterion_main!(ops);
