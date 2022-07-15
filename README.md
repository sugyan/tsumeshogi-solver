[![Rust](https://github.com/sugyan/tsumeshogi-solver/actions/workflows/rust.yml/badge.svg?branch=main)](https://github.com/sugyan/tsumeshogi-solver/actions/workflows/rust.yml)

# tsumeshogi-solver

```
% ./tsumeshogi-solver -v '9/9/3pp4/+r2k1p3/2L1+p4/2+R6/B8/B8/9 b 4g4s4n3l14p 1'
9/9/3pp4/+r2k1p3/2L1+p4/2+R6/B8/B8/9 b 4g4s4n3l14p 1:
P1 *  *  *  *  *  *  *  *  *
P2 *  *  *  *  *  *  *  *  *
P3 *  *  * -FU-FU *  *  *  *
P4-RY *  * -OU * -FU *  *  *
P5 *  * +KY * -TO *  *  *  *
P6 *  * +RY *  *  *  *  *  *
P7+KA *  *  *  *  *  *  *  *
P8+KA *  *  *  *  *  *  *  *
P9 *  *  *  *  *  *  *  *  *
P-00AL
+

Ok("7e7b+ N*8f 7f7c")
elapsed: 35.561833ms
```

### Run

```
Tsumeshogi Solver 0.6.0

USAGE:
    tsumeshogi-solver [OPTIONS] <INPUTS>...

ARGS:
    <INPUTS>...    Input files or SFEN strings

OPTIONS:
    -h, --help                      Print help information
    -i, --input-format <FORMAT>     Input format [default: sfen] [possible values: sfen, csa, kif]
    -o, --output-format <FORMAT>    Output format [default: usi] [possible values: usi, csa, kifu]
    -t, --timeout <TIMEOUT>         Time limit to solve (seconds)
    -v, --verbose                   Verbose mode
    -V, --version                   Print version information
```


### Benchmark

```
cargo +nightly bench
```
