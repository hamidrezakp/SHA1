use sha_1::*;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    let text = args[1].clone();
    let e_hash = args[2].clone();

    let text_u8 = text.into_bytes();
    let padded_message = sha_1::pad_message(&text_u8).unwrap();
    let sha1_hash = sha_1::compute(padded_message).unwrap();
    assert_eq!(e_hash, sha1_hash);
}
