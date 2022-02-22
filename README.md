[![Rust](https://github.com/sugyan/tsumeshogi-solver/actions/workflows/rust.yml/badge.svg?branch=main)](https://github.com/sugyan/tsumeshogi-solver/actions/workflows/rust.yml)

# tsumeshogi-solver

```
$ cat 3.csa
V2.2
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

$ ./tsumeshogi-solver --format csa -v 3.csa
   9   8   7   6   5   4   3   2   1
+---+---+---+---+---+---+---+---+---+
|   |   |   |   |   |   |   |   |   | a
+---+---+---+---+---+---+---+---+---+
|   |   |   |   |   |   |   |   |   | b
+---+---+---+---+---+---+---+---+---+
|   |   |   |  p|  p|   |   |   |   | c
+---+---+---+---+---+---+---+---+---+
| +r|   |   |  k|   |  p|   |   |   | d
+---+---+---+---+---+---+---+---+---+
|   |   |  L|   | +p|   |   |   |   | e
+---+---+---+---+---+---+---+---+---+
|   |   | +R|   |   |   |   |   |   | f
+---+---+---+---+---+---+---+---+---+
|  B|   |   |   |   |   |   |   |   | g
+---+---+---+---+---+---+---+---+---+
|  B|   |   |   |   |   |   |   |   | h
+---+---+---+---+---+---+---+---+---+
|   |   |   |   |   |   |   |   |   | i
+---+---+---+---+---+---+---+---+---+
Side to move: Black
Hand (Black):
Hand (White): g4 s4 n4 l3 p14
Ply: 1

["7e7b+", "9d9g", "7f7c"]
```

### Run

```
Tsumeshogi Solver 0.3

USAGE:
    tsumeshogi-solver [FLAGS] [OPTIONS] <INPUT>...

FLAGS:
    -h, --help       Prints help information
    -v, --verbose    Verbose mode
    -V, --version    Prints version information

OPTIONS:
    -f, --format <format>    input format [default: sfen]  [possible values: sfen, csa, kif]

ARGS:
    <INPUT>...    Input files or strings
```


### Benchmark

```
cargo +nightly bench
```
