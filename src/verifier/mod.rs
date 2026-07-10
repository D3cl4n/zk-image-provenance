use ff::PrimeField;
use secp256k1::{Secp256k1, Message, PublicKey, ecdsa::{Signature}};
use crate::png;


// pack a vector of u8s into field elements
fn convert_into_field_elements<F: PrimeField>(data: Vec<u8>) -> Vec<F> {
    let mut elements: Vec<F> = vec![];
    for &byte in data.iter() {
        elements.push(F::from(byte as u64));
    } 

    elements
}


// combine the hash and packed grey pixels into vector of expected field elements
pub fn construct_expected_value<F: PrimeField>(edited_img: &String) -> Vec<F> {
    let mut expected: Vec<F> = vec![];

    expected.extend(convert_into_field_elements::<F>(
        png::get_png_greyscale_values(edited_img)
    ));
    expected.extend(convert_into_field_elements::<F>(
        png::extract_chunk_data(edited_img, b"eXIf")
    ));
    
    // match circuit which exposes hash as one field element
    let hash_bytes: Vec<u8> = png::extract_chunk_data(edited_img, b"hASh");
    let mut hash_slice: [u8; 32] = hash_bytes.try_into().expect("[!] Hash chunk is not 32 bytes long");
    hash_slice.reverse(); // hASh chunk is big-endian, from_repr expects little-endian
    let mut repr = F::Repr::default();
    repr.as_mut().copy_from_slice(&hash_slice);
    let hash_element: F = Option::from(F::from_repr(repr))
        .expect("[!] Hash bytes are not a valid field element");
    expected.push(hash_element);

    println!("[*] Verifier expected len {:?}", expected.len());

    expected
}


// extract the bytes of the ECDSA signature from the edited png
fn extract_signature_from_png(edited_img: &String) -> Signature {
    println!("[*] Extracting ECDSA signature from edited png");

    // extract the hash value from the chunks
    let signature: Vec<u8> = png::extract_chunk_data(edited_img, b"sIGn");
    let signature_bytes: [u8; 64] = signature.try_into().expect("[!] Vector is not 64 elements long");

    Signature::from_compact(&signature_bytes).expect("[!] Signature is incorrect length")
}


// extract hash bytes as a slice and construct a Message struct
fn ecdsa_message_from_digest(edited_img: &String) -> Message {
    println!("[*] Extracting Poseidon hash from edited png");
    let image_chunks: Vec<png::PngChunk> = png::get_image_chunks(edited_img);

    // extract the hash value from the chunks
    let hash: Vec<u8> = png::extract_chunk_data(edited_img, b"hASh");
    let hash_slice: [u8; 32] = hash.try_into().expect("[!] Vector is not 32 elements long");

    Message::from_digest(hash_slice)
}


// load the verifying key from the .bin file
fn verifying_key_from_bin(vk_bin: &String) -> PublicKey {
    let vk_bytes = std::fs::read(vk_bin).expect("[!] Failed to read verifying key");

    PublicKey::from_slice(&vk_bytes).expect("[!] Failed to construct PublicKey")
}


// verify the ECDSA signature off-circuit given the public key
pub fn verify_ecdsa_signature(edited_img: &String, public_key: &String) {
    println!("[*] Verifying ECDSA signature");
    let message: Message = ecdsa_message_from_digest(edited_img);
    let secp = Secp256k1::new();
    let vk: PublicKey = verifying_key_from_bin(public_key);
    let signature: Signature = extract_signature_from_png(edited_img);
    let mut sig = signature;
    sig.normalize_s();

    println!("[*] Verifying key hex: {}", hex::encode(vk.serialize_uncompressed()));
    println!("[*] H being verified: {}", hex::encode(message.as_ref()));
    println!("[*] Signature hex: {}", hex::encode(signature.serialize_compact()));
    assert!(secp.verify_ecdsa(message, &sig, &vk).is_ok());
} 