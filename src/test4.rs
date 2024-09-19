fn hex_to_decimal(hex_str: &str) -> u128 {
    u128::from_str_radix(hex_str, 16).unwrap()
}

pub fn decrypted() {
    let hex1 = "3fd6eb1a7baac9d93ec927b45599ee4332d58fecb88e3df889b814685accc002";
    let hex2 = "6667c5ccb9e48bde";

    let decimal1 = hex_to_decimal(hex1);
    let decimal2 = hex_to_decimal(hex2);

    println!("3fd6eb1a7baac9d93ec927b45599ee4332d58fecb88e3df889b814685accc002 in decimal is: {}", decimal1);
    println!("6667c5ccb9e48bde in decimal is: {}", decimal2);
}
