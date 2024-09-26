#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]
use bn254_hash2curve::hash2g1::{self, ExpandMsgSHA256, Hash2FieldBN254};
use ark_bn254::fq::Fq;

fn check_hash2g1() {
    let domain = b"testing-evmbls";
    let msg = b"123";
    let point = hash2g1::HashToG1(msg, domain);
    println!("point: {:?}", point);
}

fn check_hash2field() {
    let domain = b"testing-evmbls";
    let msg = b"abc";
    let point = Fq::hash_to_field(msg, domain,2);
    println!("point: {:?}", point[0].0.to_string());
    println!("point: {:?}", point[1].0.to_string());
}

fn check_expand_message() {
    let domain = b"QUUX-V01-CS02-with-BN254G1_XMD:SHA-256_SVDW_RO_";
    let msg = b"abc";
    let point = Fq::expand_message(msg, domain, 96);
    let hex_point: String = point.iter().map(|byte| format!("{:02x}", byte)).collect();
    println!("point: {:?}", hex_point);
}

fn check_expand_message_xmd() {
    let domain = b"testing-evmbls";
    let msg = b"abc";
    let point = Fq::expand_message(msg, domain, 96);
    let hex_point: String = point.iter().map(|byte| format!("{:02x}", byte)).collect();
    println!("point: {:?}", hex_point);
}

fn main() {
    // check_hash2g1();
    check_hash2field();
    // check_expand_message();
    // check_expand_message_xmd();
}