//! Simple big integer library.
//!
//! Everything lives in `mod bigint` with no dependencies, so the whole module
//! can be copy-pasted into a single-file program (e.g. competitive programming).
pub mod bigint {
    use std::cmp::Ordering;
    use std::fmt;
    use std::fmt::Write;
    use std::ops::{
        Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Rem, RemAssign, Sub, SubAssign,
    };
    use std::str::FromStr;

    // Limbs are little-endian in base 10^9, so parsing and printing are linear.
    const BASE: u32 = 1_000_000_000;
    const BASE64: u64 = BASE as u64;
    const BASE_DIGITS: usize = 9;
    // Below this many limbs schoolbook multiplication beats Karatsuba.
    const KARATSUBA_THRESHOLD: usize = 32;

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct ParseBigIntError;

    impl fmt::Display for ParseBigIntError {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            f.write_str("invalid big integer literal")
        }
    }

    impl std::error::Error for ParseBigIntError {}

    // ---------- limb helpers (inputs and outputs have no high zero limbs) ----------

    fn trim(mut v: Vec<u32>) -> Vec<u32> {
        while v.last() == Some(&0) {
            v.pop();
        }
        v
    }

    fn trimmed(a: &[u32]) -> &[u32] {
        let mut n = a.len();
        while n > 0 && a[n - 1] == 0 {
            n -= 1;
        }
        &a[..n]
    }

    fn cmp_limbs(a: &[u32], b: &[u32]) -> Ordering {
        a.len()
            .cmp(&b.len())
            .then_with(|| a.iter().rev().cmp(b.iter().rev()))
    }

    fn add_limbs(a: &[u32], b: &[u32]) -> Vec<u32> {
        let (a, b) = if a.len() >= b.len() { (a, b) } else { (b, a) };
        let mut out = Vec::with_capacity(a.len() + 1);
        let mut carry = 0;
        for (i, &x) in a.iter().enumerate() {
            let mut s = x + b.get(i).cloned().unwrap_or(0) + carry;
            carry = if s >= BASE {
                s -= BASE;
                1
            } else {
                0
            };
            out.push(s);
        }
        if carry > 0 {
            out.push(carry);
        }
        out
    }

    /// Requires `a >= b`.
    fn sub_limbs(a: &[u32], b: &[u32]) -> Vec<u32> {
        let mut out = Vec::with_capacity(a.len());
        let mut borrow = 0;
        for (i, &x) in a.iter().enumerate() {
            let s = b.get(i).cloned().unwrap_or(0) + borrow;
            if x >= s {
                out.push(x - s);
                borrow = 0;
            } else {
                out.push(x + BASE - s);
                borrow = 1;
            }
        }
        debug_assert!(borrow == 0, "sub_limbs underflow");
        trim(out)
    }

    /// Adds `src` into `dst` starting at limb `offset`; `dst` must be long enough.
    fn add_into(dst: &mut [u32], src: &[u32], offset: usize) {
        let mut carry = 0;
        let mut i = 0;
        while i < src.len() || carry > 0 {
            let s = dst[offset + i] + src.get(i).cloned().unwrap_or(0) + carry;
            if s >= BASE {
                dst[offset + i] = s - BASE;
                carry = 1;
            } else {
                dst[offset + i] = s;
                carry = 0;
            }
            i += 1;
        }
    }

    fn schoolbook(a: &[u32], b: &[u32]) -> Vec<u32> {
        let mut out = vec![0u32; a.len() + b.len()];
        for (i, &x) in a.iter().enumerate() {
            if x == 0 {
                continue;
            }
            let mut carry = 0u64;
            for (j, &y) in b.iter().enumerate() {
                let cur = u64::from(out[i + j]) + u64::from(x) * u64::from(y) + carry;
                out[i + j] = (cur % BASE64) as u32;
                carry = cur / BASE64;
            }
            out[i + b.len()] = carry as u32;
        }
        trim(out)
    }

    fn mul_limbs(a: &[u32], b: &[u32]) -> Vec<u32> {
        let (a, b) = (trimmed(a), trimmed(b));
        if a.is_empty() || b.is_empty() {
            return Vec::new();
        }
        if a.len().min(b.len()) < KARATSUBA_THRESHOLD {
            return schoolbook(a, b);
        }
        // Karatsuba: (a1*B^m + a0)(b1*B^m + b0)
        let m = a.len().max(b.len()) / 2;
        let (a0, a1) = a.split_at(a.len().min(m));
        let (b0, b1) = b.split_at(b.len().min(m));
        let z0 = mul_limbs(a0, b0);
        let z2 = mul_limbs(a1, b1);
        let mid = mul_limbs(&add_limbs(a0, a1), &add_limbs(b0, b1));
        let z1 = sub_limbs(&sub_limbs(&mid, &z0), &z2);
        let mut out = vec![0u32; a.len() + b.len() + 1];
        add_into(&mut out, &z0, 0);
        add_into(&mut out, &z1, m);
        add_into(&mut out, &z2, 2 * m);
        trim(out)
    }

    fn mul_small(a: &[u32], f: u32) -> Vec<u32> {
        let mut out = Vec::with_capacity(a.len() + 1);
        let mut carry = 0u64;
        for &x in a {
            let cur = u64::from(x) * u64::from(f) + carry;
            out.push((cur % BASE64) as u32);
            carry = cur / BASE64;
        }
        out.push(carry as u32);
        trim(out)
    }

    /// `d` must be nonzero and fit in 32 bits.
    fn divmod_small(a: &[u32], d: u32) -> (Vec<u32>, u32) {
        let mut q = vec![0u32; a.len()];
        let mut rem = 0u64;
        for i in (0..a.len()).rev() {
            let cur = rem * BASE64 + u64::from(a[i]);
            q[i] = (cur / u64::from(d)) as u32;
            rem = cur % u64::from(d);
        }
        (trim(q), rem as u32)
    }

    /// Knuth algorithm D. `b` must be nonzero.
    fn divmod_limbs(a: &[u32], b: &[u32]) -> (Vec<u32>, Vec<u32>) {
        if cmp_limbs(a, b) == Ordering::Less {
            return (Vec::new(), a.to_vec());
        }
        if b.len() == 1 {
            let (q, r) = divmod_small(a, b[0]);
            return (q, if r == 0 { Vec::new() } else { vec![r] });
        }
        // Normalize so the divisor's top limb is >= BASE / 2.
        let f = (BASE64 / (u64::from(b[b.len() - 1]) + 1)) as u32;
        let mut u = mul_small(a, f);
        u.resize(a.len() + 1, 0);
        let v = mul_small(b, f);
        let n = v.len();
        let m = a.len() - n;
        let (vtop, vsec) = (u64::from(v[n - 1]), u64::from(v[n - 2]));
        let mut q = vec![0u32; m + 1];
        for j in (0..=m).rev() {
            let num = u64::from(u[j + n]) * BASE64 + u64::from(u[j + n - 1]);
            let mut qhat = num / vtop;
            let mut rhat = num % vtop;
            while qhat >= BASE64 || qhat * vsec > rhat * BASE64 + u64::from(u[j + n - 2]) {
                qhat -= 1;
                rhat += vtop;
                if rhat >= BASE64 {
                    break;
                }
            }
            // u[j..=j+n] -= qhat * v
            let mut borrow = 0i64;
            let mut carry = 0u64;
            for i in 0..n {
                let p = qhat * u64::from(v[i]) + carry;
                carry = p / BASE64;
                let t = i64::from(u[i + j]) - (p % BASE64) as i64 - borrow;
                if t < 0 {
                    u[i + j] = (t + BASE64 as i64) as u32;
                    borrow = 1;
                } else {
                    u[i + j] = t as u32;
                    borrow = 0;
                }
            }
            let top = i64::from(u[j + n]) - carry as i64 - borrow;
            if top < 0 {
                // qhat was one too big: add v back.
                qhat -= 1;
                let mut c = 0;
                for i in 0..n {
                    let s = u[i + j] + v[i] + c;
                    if s >= BASE {
                        u[i + j] = s - BASE;
                        c = 1;
                    } else {
                        u[i + j] = s;
                        c = 0;
                    }
                }
                u[j + n] = (top + i64::from(c)) as u32;
            } else {
                u[j + n] = top as u32;
            }
            q[j] = qhat as u32;
        }
        let (r, _) = divmod_small(trimmed(&u[..n]), f);
        (trim(q), r)
    }

    fn parse_digits(s: &str) -> Result<Vec<u32>, ParseBigIntError> {
        let bytes = s.as_bytes();
        if bytes.is_empty() || !bytes.iter().all(|c| c.is_ascii_digit()) {
            return Err(ParseBigIntError);
        }
        let mut out = Vec::with_capacity(bytes.len() / BASE_DIGITS + 1);
        let mut end = bytes.len();
        while end > 0 {
            let start = end.saturating_sub(BASE_DIGITS);
            out.push(
                bytes[start..end]
                    .iter()
                    .fold(0u32, |acc, &c| acc * 10 + u32::from(c - b'0')),
            );
            end = start;
        }
        Ok(trim(out))
    }

    // Forwards owned / mixed-reference operators and `op=` to the `&a op &b` impl.
    macro_rules! forward_binop {
        ($t:ident, $tr:ident, $m:ident, $atr:ident, $am:ident) => {
            impl $tr<$t> for $t {
                type Output = $t;
                fn $m(self, o: $t) -> $t {
                    (&self).$m(&o)
                }
            }
            impl<'a> $tr<&'a $t> for $t {
                type Output = $t;
                fn $m(self, o: &$t) -> $t {
                    (&self).$m(o)
                }
            }
            impl<'a> $tr<$t> for &'a $t {
                type Output = $t;
                fn $m(self, o: $t) -> $t {
                    self.$m(&o)
                }
            }
            impl $atr<$t> for $t {
                fn $am(&mut self, o: $t) {
                    *self = (&*self).$m(&o);
                }
            }
            impl<'a> $atr<&'a $t> for $t {
                fn $am(&mut self, o: &$t) {
                    *self = (&*self).$m(o);
                }
            }
        };
    }

    // ---------- BigUInteger ----------

    /// Non-negative big integer.
    #[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
    pub struct BigUInteger {
        limbs: Vec<u32>,
    }

    impl BigUInteger {
        pub fn zero() -> Self {
            BigUInteger { limbs: Vec::new() }
        }

        pub fn one() -> Self {
            BigUInteger { limbs: vec![1] }
        }

        /// Parses decimal digits. Panics on invalid input; use `str::parse` for a `Result`.
        #[allow(clippy::should_implement_trait)]
        pub fn from_str(s: &str) -> Self {
            s.parse().expect("invalid BigUInteger literal")
        }

        pub fn is_zero(&self) -> bool {
            self.limbs.is_empty()
        }

        pub fn checked_sub(&self, o: &Self) -> Option<Self> {
            if cmp_limbs(&self.limbs, &o.limbs) == Ordering::Less {
                None
            } else {
                Some(BigUInteger {
                    limbs: sub_limbs(&self.limbs, &o.limbs),
                })
            }
        }

        /// Quotient and remainder in one division. Panics if `o` is zero.
        pub fn div_rem(&self, o: &Self) -> (Self, Self) {
            assert!(!o.is_zero(), "attempt to divide by zero");
            let (q, r) = divmod_limbs(&self.limbs, &o.limbs);
            (BigUInteger { limbs: q }, BigUInteger { limbs: r })
        }

        pub fn pow(&self, mut exp: u32) -> Self {
            let mut base = self.clone();
            let mut acc = Self::one();
            while exp > 0 {
                if exp & 1 == 1 {
                    acc = &acc * &base;
                }
                exp >>= 1;
                if exp > 0 {
                    base = &base * &base;
                }
            }
            acc
        }

        /// `self^exp mod m`. Panics if `m` is zero.
        pub fn modpow(&self, exp: &Self, m: &Self) -> Self {
            assert!(!m.is_zero(), "modpow with zero modulus");
            let mut base = self % m;
            let mut acc = &Self::one() % m;
            // Peel 30 exponent bits per small division.
            let mut e = exp.limbs.clone();
            while !e.is_empty() {
                let (q, mut bits) = divmod_small(&e, 1 << 30);
                e = q;
                for _ in 0..30 {
                    if bits & 1 == 1 {
                        acc = &(&acc * &base) % m;
                    }
                    bits >>= 1;
                    if bits == 0 && e.is_empty() {
                        break;
                    }
                    base = &(&base * &base) % m;
                }
            }
            acc
        }

        pub fn gcd(&self, o: &Self) -> Self {
            let (mut a, mut b) = (self.clone(), o.clone());
            while !b.is_zero() {
                let r = &a % &b;
                a = b;
                b = r;
            }
            a
        }

        pub fn lcm(&self, o: &Self) -> Self {
            if self.is_zero() || o.is_zero() {
                return Self::zero();
            }
            &(self / &self.gcd(o)) * o
        }

        /// Floor square root (Newton's method).
        pub fn sqrt(&self) -> Self {
            if self.is_zero() {
                return Self::zero();
            }
            // BASE^ceil(len/2) is always >= sqrt(self).
            let mut limbs = vec![0u32; (self.limbs.len() + 1) >> 1];
            limbs.push(1);
            let mut x = BigUInteger { limbs };
            loop {
                let sum = &x + &(self / &x);
                let y = BigUInteger {
                    limbs: divmod_small(&sum.limbs, 2).0,
                };
                if y >= x {
                    return x;
                }
                x = y;
            }
        }
    }

    impl FromStr for BigUInteger {
        type Err = ParseBigIntError;
        fn from_str(s: &str) -> Result<Self, Self::Err> {
            let s = s.strip_prefix('+').unwrap_or(s);
            Ok(BigUInteger {
                limbs: parse_digits(s)?,
            })
        }
    }

    impl From<u128> for BigUInteger {
        fn from(mut n: u128) -> Self {
            let mut limbs = Vec::new();
            while n > 0 {
                limbs.push((n % u128::from(BASE)) as u32);
                n /= u128::from(BASE);
            }
            BigUInteger { limbs }
        }
    }

    macro_rules! from_unsigned {
        ($($t:ty),*) => {$(
            impl From<$t> for BigUInteger {
                fn from(n: $t) -> Self { BigUInteger::from(n as u128) }
            }
            impl From<$t> for BigInteger {
                fn from(n: $t) -> Self { BigInteger::from(BigUInteger::from(n as u128)) }
            }
        )*};
    }
    from_unsigned!(u8, u16, u32, u64, usize);

    impl Ord for BigUInteger {
        fn cmp(&self, o: &Self) -> Ordering {
            cmp_limbs(&self.limbs, &o.limbs)
        }
    }

    impl PartialOrd for BigUInteger {
        fn partial_cmp(&self, o: &Self) -> Option<Ordering> {
            Some(self.cmp(o))
        }
    }

    impl fmt::Display for BigUInteger {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            let mut s = String::with_capacity(self.limbs.len() * BASE_DIGITS + 1);
            match self.limbs.split_last() {
                None => s.push('0'),
                Some((top, rest)) => {
                    write!(s, "{}", top)?;
                    for limb in rest.iter().rev() {
                        write!(s, "{:09}", limb)?;
                    }
                }
            }
            f.pad_integral(true, "", &s)
        }
    }

    impl Add<&BigUInteger> for &BigUInteger {
        type Output = BigUInteger;
        fn add(self, o: &BigUInteger) -> BigUInteger {
            BigUInteger {
                limbs: add_limbs(&self.limbs, &o.limbs),
            }
        }
    }

    impl Sub<&BigUInteger> for &BigUInteger {
        type Output = BigUInteger;
        fn sub(self, o: &BigUInteger) -> BigUInteger {
            self.checked_sub(o).expect("attempt to subtract with overflow")
        }
    }

    impl Mul<&BigUInteger> for &BigUInteger {
        type Output = BigUInteger;
        fn mul(self, o: &BigUInteger) -> BigUInteger {
            BigUInteger {
                limbs: mul_limbs(&self.limbs, &o.limbs),
            }
        }
    }

    impl Div<&BigUInteger> for &BigUInteger {
        type Output = BigUInteger;
        fn div(self, o: &BigUInteger) -> BigUInteger {
            self.div_rem(o).0
        }
    }

    impl Rem<&BigUInteger> for &BigUInteger {
        type Output = BigUInteger;
        fn rem(self, o: &BigUInteger) -> BigUInteger {
            self.div_rem(o).1
        }
    }

    forward_binop!(BigUInteger, Add, add, AddAssign, add_assign);
    forward_binop!(BigUInteger, Sub, sub, SubAssign, sub_assign);
    forward_binop!(BigUInteger, Mul, mul, MulAssign, mul_assign);
    forward_binop!(BigUInteger, Div, div, DivAssign, div_assign);
    forward_binop!(BigUInteger, Rem, rem, RemAssign, rem_assign);

    // ---------- BigInteger ----------

    /// Signed big integer. Zero is always non-negative.
    #[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
    pub struct BigInteger {
        negative: bool,
        mag: BigUInteger,
    }

    impl BigInteger {
        fn make(negative: bool, mag: BigUInteger) -> Self {
            BigInteger {
                negative: negative && !mag.is_zero(),
                mag,
            }
        }

        pub fn zero() -> Self {
            Self::default()
        }

        pub fn one() -> Self {
            Self::from(BigUInteger::one())
        }

        /// Parses an optionally signed decimal. Panics on invalid input; use `str::parse` for a `Result`.
        #[allow(clippy::should_implement_trait)]
        pub fn from_str(s: &str) -> Self {
            s.parse().expect("invalid BigInteger literal")
        }

        pub fn is_zero(&self) -> bool {
            self.mag.is_zero()
        }

        pub fn is_negative(&self) -> bool {
            self.negative
        }

        pub fn is_positive(&self) -> bool {
            !self.negative && !self.is_zero()
        }

        /// -1, 0 or 1.
        pub fn signum(&self) -> Self {
            if self.is_zero() {
                Self::zero()
            } else {
                Self::make(self.negative, BigUInteger::one())
            }
        }

        pub fn abs(&self) -> Self {
            Self::make(false, self.mag.clone())
        }

        pub fn magnitude(&self) -> &BigUInteger {
            &self.mag
        }

        /// Truncating division (like `i64`): remainder has the sign of `self`.
        pub fn div_rem(&self, o: &Self) -> (Self, Self) {
            let (q, r) = self.mag.div_rem(&o.mag);
            (
                Self::make(self.negative != o.negative, q),
                Self::make(self.negative, r),
            )
        }

        /// Floor-style division matching `i64::div_euclid`.
        pub fn div_euclid(&self, o: &Self) -> Self {
            let (q, r) = self.div_rem(o);
            if r.negative {
                if o.negative {
                    q + Self::one()
                } else {
                    q - Self::one()
                }
            } else {
                q
            }
        }

        /// Always non-negative, matching `i64::rem_euclid`.
        pub fn rem_euclid(&self, o: &Self) -> Self {
            let r = self % o;
            if r.negative {
                Self::make(false, &o.mag - &r.mag)
            } else {
                r
            }
        }

        pub fn pow(&self, exp: u32) -> Self {
            Self::make(self.negative && exp & 1 == 1, self.mag.pow(exp))
        }

        /// `self^exp mod m` in `[0, m)`. Panics if `exp < 0` or `m <= 0`.
        pub fn modpow(&self, exp: &Self, m: &Self) -> Self {
            assert!(!exp.negative, "modpow with negative exponent");
            assert!(m.is_positive(), "modpow with non-positive modulus");
            let base = self.rem_euclid(m);
            Self::from(base.mag.modpow(&exp.mag, &m.mag))
        }

        /// Always non-negative.
        pub fn gcd(&self, o: &Self) -> Self {
            Self::from(self.mag.gcd(&o.mag))
        }

        /// Always non-negative.
        pub fn lcm(&self, o: &Self) -> Self {
            Self::from(self.mag.lcm(&o.mag))
        }

        /// Floor square root. Panics if negative.
        pub fn sqrt(&self) -> Self {
            assert!(!self.negative, "square root of negative number");
            Self::from(self.mag.sqrt())
        }

        fn add_signed(a: &Self, b: &Self, negate_b: bool) -> Self {
            let b_negative = b.negative != negate_b;
            if a.negative == b_negative {
                return Self::make(a.negative, &a.mag + &b.mag);
            }
            match a.mag.cmp(&b.mag) {
                Ordering::Less => Self::make(b_negative, &b.mag - &a.mag),
                _ => Self::make(a.negative, &a.mag - &b.mag),
            }
        }
    }

    impl FromStr for BigInteger {
        type Err = ParseBigIntError;
        fn from_str(s: &str) -> Result<Self, Self::Err> {
            let (negative, digits) = match s.strip_prefix('-') {
                Some(rest) => (true, rest),
                None => (false, s.strip_prefix('+').unwrap_or(s)),
            };
            Ok(Self::make(
                negative,
                BigUInteger {
                    limbs: parse_digits(digits)?,
                },
            ))
        }
    }

    impl From<BigUInteger> for BigInteger {
        fn from(mag: BigUInteger) -> Self {
            BigInteger {
                negative: false,
                mag,
            }
        }
    }

    impl From<u128> for BigInteger {
        fn from(n: u128) -> Self {
            Self::from(BigUInteger::from(n))
        }
    }

    macro_rules! from_signed {
        ($($t:ty),*) => {$(
            impl From<$t> for BigInteger {
                fn from(n: $t) -> Self {
                    // wrapping_abs keeps MIN correct once reinterpreted as u128.
                    let n = n as i128;
                    BigInteger::make(n < 0, BigUInteger::from(n.wrapping_abs() as u128))
                }
            }
        )*};
    }
    from_signed!(i8, i16, i32, i64, i128, isize);

    impl Ord for BigInteger {
        fn cmp(&self, o: &Self) -> Ordering {
            match (self.negative, o.negative) {
                (false, true) => Ordering::Greater,
                (true, false) => Ordering::Less,
                (false, false) => self.mag.cmp(&o.mag),
                (true, true) => o.mag.cmp(&self.mag),
            }
        }
    }

    impl PartialOrd for BigInteger {
        fn partial_cmp(&self, o: &Self) -> Option<Ordering> {
            Some(self.cmp(o))
        }
    }

    impl fmt::Display for BigInteger {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            f.pad_integral(!self.negative, "", &self.mag.to_string())
        }
    }

    impl Neg for BigInteger {
        type Output = BigInteger;
        fn neg(self) -> BigInteger {
            BigInteger::make(!self.negative, self.mag)
        }
    }

    impl Neg for &BigInteger {
        type Output = BigInteger;
        fn neg(self) -> BigInteger {
            BigInteger::make(!self.negative, self.mag.clone())
        }
    }

    impl Add<&BigInteger> for &BigInteger {
        type Output = BigInteger;
        fn add(self, o: &BigInteger) -> BigInteger {
            BigInteger::add_signed(self, o, false)
        }
    }

    impl Sub<&BigInteger> for &BigInteger {
        type Output = BigInteger;
        fn sub(self, o: &BigInteger) -> BigInteger {
            BigInteger::add_signed(self, o, true)
        }
    }

    impl Mul<&BigInteger> for &BigInteger {
        type Output = BigInteger;
        fn mul(self, o: &BigInteger) -> BigInteger {
            BigInteger::make(self.negative != o.negative, &self.mag * &o.mag)
        }
    }

    impl Div<&BigInteger> for &BigInteger {
        type Output = BigInteger;
        fn div(self, o: &BigInteger) -> BigInteger {
            BigInteger::make(self.negative != o.negative, &self.mag / &o.mag)
        }
    }

    impl Rem<&BigInteger> for &BigInteger {
        type Output = BigInteger;
        fn rem(self, o: &BigInteger) -> BigInteger {
            BigInteger::make(self.negative, &self.mag % &o.mag)
        }
    }

    forward_binop!(BigInteger, Add, add, AddAssign, add_assign);
    forward_binop!(BigInteger, Sub, sub, SubAssign, sub_assign);
    forward_binop!(BigInteger, Mul, mul, MulAssign, mul_assign);
    forward_binop!(BigInteger, Div, div, DivAssign, div_assign);
    forward_binop!(BigInteger, Rem, rem, RemAssign, rem_assign);
}
