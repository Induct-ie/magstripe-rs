use magstripe_rs::{BitStream, decoder::decode_track2};
use tracing::debug;

#[test_log::test]
fn test_decode_track2_weird(){
    // We encountered this payload in the Academy Building, Park West off of a vanderbilt nfc reader
    let payload = vec![255, 255, 255, 229, 243, 253, 235, 153, 239, 53, 192, 175, 255, 255, 240];

    let stream = BitStream::new(&payload, 116).unwrap();

    debug!("========= Decoding track2");
    let decoded1 = decode_track2(&stream, false, false, false, false, false);
    debug!("========= Decoded: {decoded1:?}");
    debug!("========= Decoding track2 inverted");
    let decoded2 = decode_track2(&stream, true, false, false, false, false);
    debug!("========= Decoded: {decoded2:?}");
    debug!("========= Decoding track2 lsb first");
    let decoded3 = decode_track2(&stream, false, true, false, false, false);
    debug!("========= Decoded: {decoded3:?}");
    debug!("========= Decoding track2 inverted lsb first");
    let decoded4 = decode_track2(&stream, true, true, false, false, false);
    debug!("========= Decoded: {decoded4:?}");

    panic!("Decoded:\n (not inverted, msb first) {decoded1:?}\n (inverted, msb first) {decoded2:?}\n (not inverted, lsb first) {decoded3:?}\n (inverted, lsb first) {decoded4:?}");
}

#[test_log::test]
fn test_decode_track2_dogpatch_normal(){
    // Dogpatch fob - should decode as 0005721443
    let payload = vec![255, 255, 255, 151, 222, 242, 135, 119, 239, 102, 4, 191, 255, 255, 255, 255, 192];

    let stream = BitStream::new(&payload, 130).unwrap();

    let decoded = decode_track2(&stream, true, false, false, false, false);
    debug!("========= Decoded: {decoded:?}");

    panic!("Decoded: {decoded:?}");
}

#[test_log::test]
fn test_decode_track2_dogpatch_weird(){
    let payload = vec![255, 255, 255, 187, 247, 223, 125, 189, 182, 237, 247, 125, 253, 255, 255, 255, 255, 255, 224];

    let stream = BitStream::new(&payload, 147).unwrap();

    let decoded = decode_track2(&stream, true, false, false, false, false);
    panic!("========= Decoded: {decoded:?}");
}
