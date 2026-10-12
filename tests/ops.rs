extern crate num_bigint_simple;

use num_bigint_simple::bigint::*;

// xorshift64, deterministic without deps.
struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
    fn i64(&mut self) -> i64 {
        // Mix magnitudes so small and limb-boundary values both show up.
        let shift = self.next() % 64;
        (self.next() >> shift) as i64 * if self.next() & 1 == 0 { 1 } else { -1 }
    }
    fn big(&mut self, max_digits: u64) -> BigUInteger {
        let len = 1 + self.next() % max_digits;
        let s: String = (0..len).map(|_| (b'0' + (self.next() % 10) as u8) as char).collect();
        BigUInteger::from_str(&s)
    }
}

fn b(n: i128) -> BigInteger {
    BigInteger::from(n)
}

#[test]
fn matches_i128() {
    let mut rng = Rng(0x9E3779B97F4A7C15);
    for _ in 0..20000 {
        let (x, y) = (rng.i64() as i128, rng.i64() as i128);
        let (bx, by) = (b(x), b(y));
        assert_eq!(&bx + &by, b(x + y), "{} + {}", x, y);
        assert_eq!(&bx - &by, b(x - y), "{} - {}", x, y);
        assert_eq!(&bx * &by, b(x * y), "{} * {}", x, y);
        assert_eq!(bx.cmp(&by), x.cmp(&y));
        assert_eq!(bx.to_string(), x.to_string());
        if y != 0 {
            assert_eq!(&bx / &by, b(x / y), "{} / {}", x, y);
            assert_eq!(&bx % &by, b(x % y), "{} % {}", x, y);
            assert_eq!(bx.div_euclid(&by), b(x.div_euclid(y)));
            assert_eq!(bx.rem_euclid(&by), b(x.rem_euclid(y)));
        }
    }
}

#[test]
fn big_division_identity() {
    let mut rng = Rng(42);
    for _ in 0..300 {
        let a = rng.big(2000);
        let d = rng.big(1200);
        if d.is_zero() {
            continue;
        }
        let (q, r) = a.div_rem(&d);
        assert!(r < d);
        assert_eq!(&(&q * &d) + &r, a);
    }
}

#[test]
fn karatsuba_matches_modular_check() {
    // Large operands go through Karatsuba; verify modulo several primes.
    let mut rng = Rng(7);
    for _ in 0..50 {
        let (x, y) = (rng.big(3000), rng.big(3000));
        let p = x.clone() * y.clone();
        for &m in &[1_000_000_007u64, 998_244_353, 4_294_967_291] {
            let m = BigUInteger::from(m);
            assert_eq!(&p % &m, &(&(&x % &m) * &(&y % &m)) % &m);
        }
        if !y.is_zero() {
            assert_eq!(&p / &y, x);
        }
    }
}

#[test]
fn factorial_100() {
    let f = (1u32..=100).fold(BigUInteger::one(), |acc, i| acc * BigUInteger::from(i));
    assert_eq!(
        f.to_string(),
        "93326215443944152681699238856266700490715968264381621468592963895217599993229915608941463976156518286253697920827223758251185210916864000000000000000000000000"
    );
}

#[test]
fn pow_sqrt_gcd_lcm_modpow() {
    let two = BigUInteger::from(2u32);
    let p = two.pow(521) - BigUInteger::one(); // Mersenne prime
    // Fermat: a^(p-1) = 1 mod p
    assert_eq!(BigUInteger::from(3u32).modpow(&(&p - &BigUInteger::one()), &p), BigUInteger::one());
    assert_eq!(BigInteger::from(-2).pow(3), b(-8));
    assert_eq!(b(-3).modpow(&b(3), &b(7)), b(1)); // -27 mod 7 = 1
    assert_eq!(b(5).modpow(&b(0), &b(1)), b(0));

    let n = BigUInteger::from_str("123456789012345678901234567890123456789");
    let s = n.sqrt();
    assert!(&s * &s <= n && &(&s + &BigUInteger::one()) * &(&s + &BigUInteger::one()) > n);
    for i in 0u32..200 {
        let k = BigUInteger::from(i);
        assert_eq!((&k * &k).sqrt(), k);
    }

    assert_eq!(b(-12).gcd(&b(18)), b(6));
    assert_eq!(b(4).lcm(&b(-6)), b(12));
    assert_eq!(b(0).gcd(&b(5)), b(5));
}

#[test]
fn parse_and_format() {
    assert_eq!(BigInteger::from_str("-0"), BigInteger::zero());
    assert_eq!(BigInteger::from_str("+000123").to_string(), "123");
    assert_eq!(BigInteger::from_str("-1000000000").to_string(), "-1000000000");
    assert!("".parse::<BigInteger>().is_err());
    assert!("-".parse::<BigInteger>().is_err());
    assert!("12a".parse::<BigInteger>().is_err());
    assert_eq!(format!("{:>6}", b(-42)), "   -42");
    assert_eq!(b(i128::MIN).to_string(), i128::MIN.to_string());
    assert_eq!(-b(5), b(-5));
    let mut x = b(10);
    x += b(5);
    x *= &b(-2);
    x -= BigInteger::from(1i32);
    assert_eq!(x, b(-31));
}
