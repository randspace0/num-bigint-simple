# Simple Rust Big Integer

A simple Rust Big Integer library in one file. Easy to use and to copy-paste.

I intent to made this **Simple Rust Big Integer** to be used on competitive programming (Especially for solving [Aizu Online Judge - Library of Number Theory](http://judge.u-aizu.ac.jp/onlinejudge/finder.jsp?course=NTL)) and algorithm learning.

## How to use

Copy the whole `pub mod bigint { ... }` block from [src/lib.rs](src/lib.rs) into your file. No dependencies.

```rust
// Paste simple rust big integer library in the code

use bigint::{BigInteger, BigUInteger};

fn main() {
    let a = BigInteger::from_str("-123456789012345678901234567890");
    let b: BigInteger = "987654321".parse().unwrap();
    let c = BigInteger::from(42i64);

    println!("{}", &a + &b);          // operators work on owned values and references
    println!("{}", &a * &b - &c);
    println!("{} {}", &a / &b, &a % &b); // truncating, like i64
    println!("{}", a.rem_euclid(&b));   // always >= 0
    println!("{}", c.pow(100));
    println!("{}", c.modpow(&b, &BigInteger::from(1_000_000_007)));
    println!("{} {}", a.gcd(&b), a.abs().sqrt());

    let mut x = BigUInteger::one();
    for i in 1u32..=100 {
        x *= BigUInteger::from(i);
    }
    println!("100! = {}", x);
}
```

## Features

- `BigInteger` (signed) and `BigUInteger` (unsigned), base 10^9 limbs
- `+ - * / %` and `+= -= *= /= %=`, for owned values and references; unary `-`
- Karatsuba multiplication for large operands, Knuth long division
- `pow`, `modpow`, `gcd`, `lcm`, `sqrt` (floor), `abs`, `signum`, `div_euclid`, `rem_euclid`, `div_rem`
- `Eq`, `Ord`, `Hash`, `Display` (respects width/padding), `FromStr`, `From<primitive ints>`

Panics match primitive integers: division by zero, `BigUInteger` subtraction underflow.

## License

Licensed under [MIT License](LICENSE)
