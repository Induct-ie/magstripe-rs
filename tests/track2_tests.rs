use magstripe_rs::{decoder::decode_track2, BitStream};
use tracing::debug;

#[test_log::test]
fn test_decode_track2_weird() {
    // We encountered this payload in the Academy Building, Park West off of a vanderbilt nfc reader
    let payload = vec![
        255, 255, 255, 229, 243, 253, 235, 153, 239, 53, 192, 175, 255, 255, 240,
    ];

    let stream = BitStream::new(&payload, 116).unwrap();

    let decoded = decode_track2(&stream, true, true, false, false, false);

    assert_eq!(decoded, Ok("0100231132".to_string()));
}

#[test_log::test]
fn test_decode_track2_dogpatch_normal() {
    // Dogpatch fob - should decode as 0005721443
    let payload = vec![
        255, 255, 255, 151, 222, 242, 135, 119, 239, 102, 4, 191, 255, 255, 255, 255, 192,
    ];

    let stream = BitStream::new(&payload, 130).unwrap();

    let decoded = decode_track2(&stream, true, true, false, false, false);
    debug!("========= Decoded: {decoded:?}");

    assert_eq!(decoded, Ok("0005721443".to_string()));
}

#[test_log::test]
fn test_decode_track2_dogpatch_weird(){
    let payload = vec![255, 255, 255, 187, 247, 223, 125, 189, 182, 237, 247, 125, 253, 255, 255, 255, 255, 255, 224];

    let stream = BitStream::new(&payload, 147).unwrap();

    let decoded = decode_track2(&stream, false, true, false, false, false);
    panic!("========= Decoded: {decoded:?}");
}
