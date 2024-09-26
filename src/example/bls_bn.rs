#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]
use core::slice;
use std::ops::Neg;
use num_bigint::BigUint;
use bn254::{PrivateKey, PublicKey, ECDSA, hash_to_try_and_increment};
use k256::{elliptic_curve::bigint::Encoding, U256};
use substrate_bn::{Group, G2};
use rand_core::OsRng;

fn check_const() {
    let private_key1 = PrivateKey::random(&mut OsRng);
    let private_key2 = PrivateKey::try_from(123u64.to_be_bytes().to_vec().as_slice()).unwrap();
    let public_key1 = PublicKey::from_private_key(&private_key1);
    let public_key2 = PublicKey::from_private_key(&private_key2);
    println!("private_key1: {:?}", hex::encode(private_key1.to_bytes().unwrap()));
    println!("public_key1: {:?}", hex::encode(public_key1.to_compressed().unwrap()));
    println!("public_key1.x: {:?}", public_key1.0.x());
    println!("public_key1.y: {:?}", public_key1.0.y());
    println!("public_key1.z: {:?}", public_key1.0.z());
    println!("--------------------------------------------------------");
    println!("G2::one().x(): {:?}", G2::one().x());
    println!("G2::one().y(): {:?}", G2::one().y());
    println!("G2::one().z(): {:?}", G2::one().z());
    println!("G2::one().x().real(): {:?}", G2::one().x().real());
    println!("G2::one().y().real(): {:?}", G2::one().y().real());
    println!("G2::one().z().real(): {:?}", G2::one().z().real());
    println!("--------------------------------------------------------");
    let mut slice = [0u8; 32];
    G2::one().neg().x().real().to_big_endian(&mut slice).unwrap();
    println!("G2::one().neg().x().real(): {:?}", hex::encode(slice));
    println!("G2::onw().neg().x().real() U256: {}", U256::from_be_slice(&slice));
    println!("G2::onw().neg().x().real() uin256: {}", BigUint::from_bytes_be(&slice));
    G2::one().neg().x().imaginary().to_big_endian(&mut slice).unwrap();
    println!("G2::one().neg().x().imaginary(): {:?}", hex::encode(slice));
    println!("G2::onw().neg().x().imaginary() U256: {:?}", U256::from_be_slice(&slice));
    println!("G2::onw().neg().x().imaginary() uin256: {}", BigUint::from_bytes_be(&slice));
    G2::one().neg().y().real().to_big_endian(&mut slice).unwrap();
    println!("G2::one().neg().y().real(): {:?}", hex::encode(slice));
    println!("G2::onw().neg().y().real() U256: {:?}", U256::from_be_slice(&slice));
    println!("G2::onw().neg().y().real() uin256: {}", BigUint::from_bytes_be(&slice));
    G2::one().neg().y().imaginary().to_big_endian(&mut slice).unwrap();
    println!("G2::one().neg().y().imaginary(): {:?}", hex::encode(slice));
    println!("G2::onw().neg().y().imaginary() U256: {:?}", U256::from_be_slice(&slice));
    println!("G2::onw().neg().y().imaginary() uin256: {}", BigUint::from_bytes_be(&slice));
    G2::one().neg().z().real().to_big_endian(&mut slice).unwrap();
    println!("G2::one().neg().z().real(): {:?}", hex::encode(slice));
    println!("G2::onw().neg().z().real() U256: {:?}", U256::from_be_slice(&slice));
    println!("G2::onw().neg().z().real() uin256: {}", BigUint::from_bytes_be(&slice));
    G2::one().neg().z().imaginary().to_big_endian(&mut slice).unwrap();
    println!("G2::one().neg().z().imaginary(): {:?}", hex::encode(slice));
    println!("G2::onw().neg().z().imaginary() U256: {:?}", U256::from_be_slice(&slice));
    println!("G2::onw().neg().z().imaginary() uin256: {}", BigUint::from_bytes_be(&slice));
    println!("--------------------------------------------------------");
}

fn check_secret_key() {
    let private_key = PrivateKey::try_from(U256::from(2u64).to_be_bytes().to_vec().as_slice()).unwrap();
    println!("private_key: {:?}", hex::encode(private_key.to_bytes().unwrap()));
    let public_key = PublicKey::from_private_key(&private_key);
    println!("public_key: {:?}", hex::encode(public_key.to_compressed().unwrap()));
    let mut slice = [0u8; 32];
    public_key.0.x().real().to_big_endian(&mut slice).unwrap();
    println!("public_key.x.real: {:?}", BigUint::from_bytes_be(&slice));
    public_key.0.x().imaginary().to_big_endian(&mut slice).unwrap();
    println!("public_key.x.imaginary: {:?}", BigUint::from_bytes_be(&slice));
    public_key.0.y().real().to_big_endian(&mut slice).unwrap();
    println!("public_key.y.real: {:?}", BigUint::from_bytes_be(&slice));
    public_key.0.y().imaginary().to_big_endian(&mut slice).unwrap();
    println!("public_key.y.imaginary: {:?}", BigUint::from_bytes_be(&slice));
    public_key.0.z().real().to_big_endian(&mut slice).unwrap();
    println!("public_key.z.real: {:?}", BigUint::from_bytes_be(&slice));
    public_key.0.z().imaginary().to_big_endian(&mut slice).unwrap();
    println!("public_key.z.imaginary: {:?}", BigUint::from_bytes_be(&slice));
}

fn check_message() {
    let private_key = PrivateKey::try_from(U256::from(2u64).to_be_bytes().to_vec().as_slice()).unwrap();
    println!("private_key: {:?}", hex::encode(private_key.to_bytes().unwrap()));
    let mut public_key = PublicKey::from_private_key(&private_key);
    public_key.0.normalize(); // 必须正则化, 将元素z的值乘到x和y上, 才能进行后续操作
    let mut slice = [0u8; 32];
    public_key.0.x().real().to_big_endian(&mut slice).unwrap();
    println!("public_key.x.real: {:?}", BigUint::from_bytes_be(&slice));
    public_key.0.x().imaginary().to_big_endian(&mut slice).unwrap();
    println!("public_key.x.imaginary: {:?}", BigUint::from_bytes_be(&slice));
    public_key.0.y().real().to_big_endian(&mut slice).unwrap();
    println!("public_key.y.real: {:?}", BigUint::from_bytes_be(&slice));
    public_key.0.y().imaginary().to_big_endian(&mut slice).unwrap();
    println!("public_key.y.imaginary: {:?}", BigUint::from_bytes_be(&slice));
    let message = b"123";
    let hash_point = hash_to_try_and_increment(message).unwrap();
    // hash_point.normalize();
    hash_point.x().to_big_endian(&mut slice).unwrap();
    println!("message.x(): {:?}", BigUint::from_bytes_be(&slice));
    hash_point.y().to_big_endian(&mut slice).unwrap();
    println!("message.y(): {:?}", BigUint::from_bytes_be(&slice));
    hash_point.z().to_big_endian(&mut slice).unwrap();
    println!("message.z(): {:?}", BigUint::from_bytes_be(&slice));
    let mut signature = ECDSA::sign(message, &private_key).unwrap();
    signature.0.normalize();
    signature.0.x().to_big_endian(&mut slice).unwrap();
    println!("signature.0.x(): {:?}", BigUint::from_bytes_be(&slice));
    signature.0.y().to_big_endian(&mut slice).unwrap();
    println!("signature.0.y(): {:?}", BigUint::from_bytes_be(&slice));
    signature.0.z().to_big_endian(&mut slice).unwrap();
    println!("signature.0.z(): {:?}", BigUint::from_bytes_be(&slice));
}

fn check_calculation() {
    let p1 = G2::one();
    let p2 = G2::one();
    let mut p3 = p1 + p2;
    p3.normalize();
    let mut slice = [0u8; 32];
    p3.x().real().to_big_endian(&mut slice).unwrap();
    println!("p3.x().real(): {:?}", BigUint::from_bytes_be(&slice));
    p3.x().imaginary().to_big_endian(&mut slice).unwrap();
    println!("p3.x().imaginary(): {:?}", BigUint::from_bytes_be(&slice));
    p3.y().real().to_big_endian(&mut slice).unwrap();
    println!("p3.y().real(): {:?}", BigUint::from_bytes_be(&slice));
    p3.y().imaginary().to_big_endian(&mut slice).unwrap();
    println!("p3.y().imaginary(): {:?}", BigUint::from_bytes_be(&slice));
}

fn check_double() {
    let (x, y, z) = (2, 3, 1);
    let a = x * x;
    let b = y * y;
    let c = b * b;
    let mut d = (x + b) * (x + b) - a - c;
    d = d + d;
    let e = a + a + a;
    let f = e * e;
    let x3 = f - (d + d);
    let mut eight_c = c + c;
    eight_c = eight_c + eight_c;
    eight_c = eight_c + eight_c;
    let y1z1 = y * z;
    let x1 = x3*2*y;
    let y1 = e * (d - x3) - eight_c;
    let z1 = y1z1 + y1z1;
    println!("x1, y1, z1: {}, {}, {}", x1, y1, z1);
}

fn check_aggregate() {
    let private_key1 = PrivateKey::try_from(U256::from(1u64).to_be_bytes().to_vec().as_slice()).unwrap();
    let mut public_key1 = PublicKey::from_private_key(&private_key1);
    public_key1.0.normalize();
    let private_key2 = PrivateKey::try_from(U256::from(2u64).to_be_bytes().to_vec().as_slice()).unwrap();
    let mut public_key2 = PublicKey::from_private_key(&private_key2);
    public_key2.0.normalize();
    let private_key3 = PrivateKey::try_from(U256::from(3u64).to_be_bytes().to_vec().as_slice()).unwrap();
    let mut public_key3 = PublicKey::from_private_key(&private_key3);
    public_key3.0.normalize();
    
    let mut aggregated_pubkey = public_key1 + public_key2 + public_key3;
    aggregated_pubkey.0.normalize();
    let mut slice = [0u8; 32];
    aggregated_pubkey.0.x().real().to_big_endian(&mut slice).unwrap();
    println!("aggregated_pubkey.x.real: {:?}", BigUint::from_bytes_be(&slice));
    aggregated_pubkey.0.x().imaginary().to_big_endian(&mut slice).unwrap();
    println!("aggregated_pubkey.x.imaginary: {:?}", BigUint::from_bytes_be(&slice));
    aggregated_pubkey.0.y().real().to_big_endian(&mut slice).unwrap();
    println!("aggregated_pubkey.y.real: {:?}", BigUint::from_bytes_be(&slice));
    aggregated_pubkey.0.y().imaginary().to_big_endian(&mut slice).unwrap();
    println!("aggregated_pubkey.y.imaginary: {:?}", BigUint::from_bytes_be(&slice));

    let message = b"123";
    let hash_point = hash_to_try_and_increment(message).unwrap();
    hash_point.x().to_big_endian(&mut slice).unwrap();
    println!("message.x(): {:?}", BigUint::from_bytes_be(&slice));
    hash_point.y().to_big_endian(&mut slice).unwrap();
    println!("message.y(): {:?}", BigUint::from_bytes_be(&slice));
    hash_point.z().to_big_endian(&mut slice).unwrap();
    println!("message.z(): {:?}", BigUint::from_bytes_be(&slice));

    let mut signature1 = ECDSA::sign(&message, &private_key1).unwrap();
    signature1.0.normalize();
    let mut signature2 = ECDSA::sign(&message, &private_key2).unwrap();
    signature2.0.normalize();
    let mut signature3 = ECDSA::sign(&message, &private_key3).unwrap();
    signature3.0.normalize();

    let mut aggregated_signature = signature1 + signature2 + signature3;
    aggregated_signature.0.normalize();
    aggregated_signature.0.x().to_big_endian(&mut slice).unwrap();
    println!("signature.0.x(): {:?}", BigUint::from_bytes_be(&slice));
    aggregated_signature.0.y().to_big_endian(&mut slice).unwrap();
    println!("signature.0.y(): {:?}", BigUint::from_bytes_be(&slice));
    aggregated_signature.0.z().to_big_endian(&mut slice).unwrap();
    println!("signature.0.z(): {:?}", BigUint::from_bytes_be(&slice));

}

fn main() {
    // check_const();
    // check_secret_key();
    // check_message();
    // check_calculation();
    // check_double();
    check_aggregate();
    

    // let private_key2 = PrivateKey::random(&mut OsRng);
    // let public_key2 = PublicKey::from_private_key(&private_key2);
    // println!("private_key2: {:?}", hex::encode(private_key2.to_bytes().unwrap()));
    // println!("public_key2: {:?}", hex::encode(public_key2.to_compressed().unwrap()));

    // let private_key3 = PrivateKey::random(&mut OsRng);
    // let public_key3 = PublicKey::from_private_key(&private_key3);
    // println!("private_key3: {:?}", hex::encode(private_key3.to_bytes().unwrap()));
    // println!("public_key3: {:?}", hex::encode(public_key3.to_compressed().unwrap()));
    // let message = b"Hello, BLS!";
    
    // let signature1 = ECDSA::sign(&message, &private_key1).unwrap();
    // let signature2 = ECDSA::sign(&message, &private_key2).unwrap();
    // let signature3 = ECDSA::sign(&message, &private_key3).unwrap();
    
    // let aggregated_pubkey = public_key1 + public_key2 + public_key3;
    // let aggregated_signature = signature1 + signature2 + signature3;

    // let aggregated_pubkey_compressed = hex::encode(aggregated_pubkey.to_compressed().unwrap());
    // let aggregated_signature_compressed = hex::encode(aggregated_signature.to_compressed().unwrap());
    // println!("aggregated_pubkey: {:?}, {}", aggregated_pubkey_compressed, aggregated_pubkey_compressed.len());
    // println!("aggregated_signature: {:?}, {}", aggregated_signature_compressed, aggregated_signature_compressed.len());

    // if let Ok(()) = ECDSA::verify(&message, &aggregated_signature, &aggregated_pubkey) {
    //     println!("Signature is valid");
    // } else {
    //     println!("Signature is invalid");
    // }
}