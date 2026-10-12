extern crate num_bigint_simple;

use num_bigint_simple::bigint::*;

#[test]
fn print() {
    println!("{}", BigInteger::from_str("-100"));
    println!("{}", BigInteger::from_str("+100"));
    println!("{}", BigInteger::from_str("100"));

    let num_l = [String::from("420"), String::from("-69")];

    println!(
        "{} + {} = {}",
        BigInteger::from_str(&num_l[0]),
        BigInteger::from_str(&num_l[1]),
        BigInteger::from_str(&num_l[0]) + BigInteger::from_str(&num_l[1])
    );
    println!(
        "{} - {} = {}",
        BigInteger::from_str(&num_l[0]),
        BigInteger::from_str(&num_l[1]),
        BigInteger::from_str(&num_l[0]) - BigInteger::from_str(&num_l[1])
    );
}
