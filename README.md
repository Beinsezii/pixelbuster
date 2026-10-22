# Pixelbuster 0.2.3
A pixel math library because I can

## Features
  * Tiny core in pure Rust
    * FFI available
    * Fast, works directly on a mutable pointer
  * Simple 100% easy to understand scripting language that definitely will not give you assembly PTSD
    * Kind of turing complete maybe
    * a whole 18 variables you can use, currently limited by the technology of our time.

## Implementations
  * [GIMP plugin here](https://github.com/Beinsezii/bsz-gimp-plugins)
  * a simple GUI can be built with `--features=gui` or found in the releases

## Goals for 1.0
  * SIMD
  * Docs
  * Non-alpha
  * should probably have more than 18 vars
  * the gui looks like it was written by a 14 y/o Minecraft modder
  
## Usage
See the HELP either in [code](./src/lib.rs#L10) or ui for language reference

### As a library
Add this git to `Cargo.toml` and go for it. Basically nothing is documented as most of this is written at around 2 am, but I believe in you nontheless.

There's currently 3 fns available via FFI:

`void pixelbuster_ffi(char* code, char* channels, float* pixels, pixels_len: unsigned int, width: unsigned int);`
  * `code:` Null-terminated UTF-8 string with lines of code
  * `channels:` Null-terminated UTF-8 string with pixel format as letters. Ex: "rgba"
  * `pixels:` Is cast into float* when validated; can point to raw bytes as well
    * Currently must *always* have a 4th channel for alpha
  * `pixels_size:` Size of `pixels` in bytes
  * `width:` Width of image in pixels. Set to 0 if unkown.

`void pixelbuster_ffi_ext(... float e1..float e9);`
 * Same as `pixelbuster_ffi()` with 9 extra floats at the end of the signature to fill out the external variables

`char* pb_help_ffi();`
Simply returns a null-terminated UTF-8 string with HELP

### GUI/GIMP
<img width=300 src="./src/bin/gui/screenshot.png"/>

 1) Type code
 2) Press "update"
 3) Impress mates

### As a weapon to flex your intelligence on inferior species
```
# Mandlebrot viewer, z^2 + c
#
# Defaults, comment out if using sliders
# max iterations
e1 = 95
# c center real
e2 = -0.75
# c center imag
e3 = 0
# c scale / world width
e4 = 2.5
# z initial real
e5 = 0
# z initial imag
e6 = 0

# factor = 8/(2*e1-1)
e9 = e1; e9 += e1; e9 -= 1; e9 recip e9; e9 *= 8

# aspect correction
v8 = height; v8 /= width

# c = pixel-derived complex constant
# real
v3 = xnorm; v3 -= 0.5; v3 *= e4; v3 += e2
# imaginary
v4 = 0.5; v4 -= ynorm; v4 *= e4; v4 *= v8; v4 += e3

# initial z
v1 = e5
v2 = e6

# initialize z^2 registers
v5 = v1; v5 *= v1
v6 = v2; v6 *= v2

:loop

# tmp = 2 * z_re * z_im
v9 = v1; v9 *= v2; v9 += v9

# z_re = z_re^2 - z_im^2 + c_re
v1 = v5; v1 -= v6; v1 += v3

# z_im = tmp + c_im
v2 = v9; v2 += v4

# count++
v7 += 1

# refresh z^2 registers for next iteration
v5 = v1; v5 *= v1
v6 = v2; v6 *= v2

# stop = max(z_re^2 + z_im^2, count * factor)
v9 = v5; v9 += v6; v8 = v7; v8 *= e9; v8 max v9

if v8 <= 4 jmp loop

:done

oklch

# lightness = 1 - sqrt((max_iterations - count) / max_iterations)
l = e1; l -= v7; l /= e1; l sqrt l; l invert 1

# stable regions black
if l >= 1 l = 0

# 20% chroma hue intensity map
c = 0.2
h = l; h * 270
```

## F.A.Q.
Question|Answer
---|---
Couldn't help but notice there's no way to do any complex filters such as a blur. What the hell even is the point of this|Each pixel only has knowledge of itself, as the core operates directly on a provided buffer instead of writing to a 'shadow' buffer then committing changes. Use G'MIC or something idk.
It crashed!|Yup.
