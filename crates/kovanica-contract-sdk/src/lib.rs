#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

pub mod storage;

pub use storage::{decode, encode, encode_error, storage_key};

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::string::String;
    use alloc::vec;
    use alloc::vec::Vec;
    use kovanica_common::ContractError;
    use serde::{Deserialize, Serialize};

    #[test]
    fn test_abi_primitives_canonical_vectors_and_roundtrip() {
        // --- bool ---
        let false_vec = [0x00u8];
        let true_vec = [0x01u8];
        assert_eq!(encode(&false).unwrap(), false_vec);
        assert_eq!(encode(&true).unwrap(), true_vec);
        assert!(!decode::<bool>(&false_vec).unwrap());
        assert!(decode::<bool>(&true_vec).unwrap());

        // --- u8 ---
        assert_eq!(encode(&0x00u8).unwrap(), [0x00]);
        assert_eq!(encode(&0x5Au8).unwrap(), [0x5A]);
        assert_eq!(encode(&0xFFu8).unwrap(), [0xFF]);
        assert_eq!(decode::<u8>(&[0x5A]).unwrap(), 0x5Au8);
        assert_eq!(decode::<u8>(&encode(&0x5Au8).unwrap()).unwrap(), 0x5Au8);

        // --- u16 ---
        let u16_val = 0x1234u16;
        let expected_u16 = [0x34, 0x12];
        assert_eq!(encode(&u16_val).unwrap(), expected_u16);
        assert_eq!(decode::<u16>(&expected_u16).unwrap(), u16_val);
        assert_eq!(decode::<u16>(&encode(&u16_val).unwrap()).unwrap(), u16_val);
        assert_eq!(encode(&0u16).unwrap(), [0x00, 0x00]);
        assert_eq!(encode(&u16::MAX).unwrap(), [0xFF, 0xFF]);

        // --- u32 ---
        let u32_val = 0xDEADBEEFu32;
        let expected_u32 = [0xEF, 0xBE, 0xAD, 0xDE];
        assert_eq!(encode(&u32_val).unwrap(), expected_u32);
        assert_eq!(decode::<u32>(&expected_u32).unwrap(), u32_val);
        assert_eq!(decode::<u32>(&encode(&u32_val).unwrap()).unwrap(), u32_val);
        assert_eq!(encode(&0u32).unwrap(), [0x00, 0x00, 0x00, 0x00]);
        assert_eq!(encode(&u32::MAX).unwrap(), [0xFF, 0xFF, 0xFF, 0xFF]);

        // --- u64 ---
        let u64_val = 1_000_000_000u64;
        let expected_u64 = [0x00, 0xCA, 0x9A, 0x3B, 0x00, 0x00, 0x00, 0x00];
        assert_eq!(encode(&u64_val).unwrap(), expected_u64);
        assert_eq!(decode::<u64>(&expected_u64).unwrap(), u64_val);
        assert_eq!(decode::<u64>(&encode(&u64_val).unwrap()).unwrap(), u64_val);
        assert_eq!(encode(&0u64).unwrap(), [0x00; 8]);
        assert_eq!(encode(&u64::MAX).unwrap(), [0xFF; 8]);

        // --- u128 ---
        let u128_val = 2_500_000_000u128; // 2.5 KVNC in base units (0x9502F900)
        let expected_u128 = [
            0x00, 0xF9, 0x02, 0x95, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00,
        ];
        assert_eq!(encode(&u128_val).unwrap(), expected_u128);
        assert_eq!(decode::<u128>(&expected_u128).unwrap(), u128_val);
        assert_eq!(
            decode::<u128>(&encode(&u128_val).unwrap()).unwrap(),
            u128_val
        );
        assert_eq!(encode(&0u128).unwrap(), [0x00; 16]);
        assert_eq!(encode(&u128::MAX).unwrap(), [0xFF; 16]);

        // --- i8 ---
        assert_eq!(encode(&0i8).unwrap(), [0x00]);
        assert_eq!(encode(&(-1i8)).unwrap(), [0xFF]);
        assert_eq!(encode(&(-5i8)).unwrap(), [0xFB]);
        assert_eq!(encode(&i8::MIN).unwrap(), [0x80]);
        assert_eq!(encode(&i8::MAX).unwrap(), [0x7F]);
        assert_eq!(decode::<i8>(&[0xFB]).unwrap(), -5i8);
        assert_eq!(decode::<i8>(&encode(&(-5i8)).unwrap()).unwrap(), -5i8);

        // --- i16 ---
        let i16_val = -1000i16;
        let expected_i16 = [0x18, 0xFC];
        assert_eq!(encode(&i16_val).unwrap(), expected_i16);
        assert_eq!(decode::<i16>(&expected_i16).unwrap(), i16_val);
        assert_eq!(decode::<i16>(&encode(&i16_val).unwrap()).unwrap(), i16_val);
        assert_eq!(encode(&0i16).unwrap(), [0x00, 0x00]);
        assert_eq!(encode(&(-1i16)).unwrap(), [0xFF, 0xFF]);
        assert_eq!(encode(&i16::MIN).unwrap(), [0x00, 0x80]);
        assert_eq!(encode(&i16::MAX).unwrap(), [0xFF, 0x7F]);

        // --- i32 ---
        let i32_val = -12345678i32;
        let expected_i32 = [0xB2, 0x9E, 0x43, 0xFF];
        assert_eq!(encode(&i32_val).unwrap(), expected_i32);
        assert_eq!(decode::<i32>(&expected_i32).unwrap(), i32_val);
        assert_eq!(decode::<i32>(&encode(&i32_val).unwrap()).unwrap(), i32_val);
        assert_eq!(encode(&0i32).unwrap(), [0x00; 4]);
        assert_eq!(encode(&(-1i32)).unwrap(), [0xFF; 4]);
        assert_eq!(encode(&i32::MIN).unwrap(), [0x00, 0x00, 0x00, 0x80]);
        assert_eq!(encode(&i32::MAX).unwrap(), [0xFF, 0xFF, 0xFF, 0x7F]);

        // --- i64 ---
        let i64_val = -42i64;
        let expected_i64 = [0xD6, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF];
        assert_eq!(encode(&i64_val).unwrap(), expected_i64);
        assert_eq!(decode::<i64>(&expected_i64).unwrap(), i64_val);
        assert_eq!(decode::<i64>(&encode(&i64_val).unwrap()).unwrap(), i64_val);
        assert_eq!(encode(&0i64).unwrap(), [0x00; 8]);
        assert_eq!(encode(&(-1i64)).unwrap(), [0xFF; 8]);
        assert_eq!(
            encode(&i64::MIN).unwrap(),
            [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x80]
        );
        assert_eq!(
            encode(&i64::MAX).unwrap(),
            [0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x7F]
        );

        // --- i128 ---
        let i128_val = -100i128;
        let expected_i128 = [
            0x9C, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
            0xFF, 0xFF,
        ];
        assert_eq!(encode(&i128_val).unwrap(), expected_i128);
        assert_eq!(decode::<i128>(&expected_i128).unwrap(), i128_val);
        assert_eq!(
            decode::<i128>(&encode(&i128_val).unwrap()).unwrap(),
            i128_val
        );
        assert_eq!(encode(&0i128).unwrap(), [0x00; 16]);
        assert_eq!(encode(&(-1i128)).unwrap(), [0xFF; 16]);
        let mut min_i128 = [0x00; 16];
        min_i128[15] = 0x80;
        assert_eq!(encode(&i128::MIN).unwrap(), min_i128);
        let mut max_i128 = [0xFF; 16];
        max_i128[15] = 0x7F;
        assert_eq!(encode(&i128::MAX).unwrap(), max_i128);
    }

    #[test]
    fn test_abi_strings_and_byte_vectors_canonical_vectors_and_roundtrip() {
        // --- String empty ---
        let empty_str = String::from("");
        let expected_empty = [0x00u8; 8];
        assert_eq!(encode(&empty_str).unwrap(), expected_empty);
        assert_eq!(decode::<String>(&expected_empty).unwrap(), empty_str);

        // --- String "KVNC" ---
        let kvnc_str = String::from("KVNC");
        let expected_kvnc = [
            0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, b'K', b'V', b'N', b'C',
        ];
        assert_eq!(encode(&kvnc_str).unwrap(), expected_kvnc);
        assert_eq!(decode::<String>(&expected_kvnc).unwrap(), kvnc_str);
        assert_eq!(
            decode::<String>(&encode(&kvnc_str).unwrap()).unwrap(),
            kvnc_str
        );

        // --- String UTF-8 multi-byte "Kovanica 🪙" ---
        let utf8_str = String::from("Kovanica 🪙");
        let mut expected_utf8 = vec![13, 0, 0, 0, 0, 0, 0, 0];
        expected_utf8.extend_from_slice("Kovanica 🪙".as_bytes());
        assert_eq!(encode(&utf8_str).unwrap(), expected_utf8);
        assert_eq!(decode::<String>(&expected_utf8).unwrap(), utf8_str);

        // --- Vec<u8> empty ---
        let empty_vec: Vec<u8> = vec![];
        let expected_empty_vec = [0x00u8; 8];
        assert_eq!(encode(&empty_vec).unwrap(), expected_empty_vec);
        assert_eq!(decode::<Vec<u8>>(&expected_empty_vec).unwrap(), empty_vec);

        // --- Vec<u8> non-empty ---
        let byte_vec: Vec<u8> = vec![0xAA, 0xBB, 0xCC];
        let expected_byte_vec = [
            0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xAA, 0xBB, 0xCC,
        ];
        assert_eq!(encode(&byte_vec).unwrap(), expected_byte_vec);
        assert_eq!(decode::<Vec<u8>>(&expected_byte_vec).unwrap(), byte_vec);
        assert_eq!(
            decode::<Vec<u8>>(&encode(&byte_vec).unwrap()).unwrap(),
            byte_vec
        );

        // --- Fixed byte array [u8; 32] ---
        let arr32 = [0x5Au8; 32];
        assert_eq!(encode(&arr32).unwrap(), arr32.to_vec());
        assert_eq!(decode::<[u8; 32]>(&arr32).unwrap(), arr32);
        assert_eq!(decode::<[u8; 32]>(&encode(&arr32).unwrap()).unwrap(), arr32);

        // --- Fixed byte array [u8; 20] ---
        let arr20 = [0x7Bu8; 20];
        assert_eq!(encode(&arr20).unwrap(), arr20.to_vec());
        assert_eq!(decode::<[u8; 20]>(&arr20).unwrap(), arr20);
        assert_eq!(decode::<[u8; 20]>(&encode(&arr20).unwrap()).unwrap(), arr20);
    }

    #[test]
    fn test_abi_options_and_tuples_canonical_vectors_and_roundtrip() {
        // --- Option<u64> ---
        let none_u64: Option<u64> = None;
        assert_eq!(encode(&none_u64).unwrap(), [0x00]);
        assert_eq!(decode::<Option<u64>>(&[0x00]).unwrap(), None);

        let some_u64: Option<u64> = Some(1_000_000_000);
        let expected_some_u64 = [0x01, 0x00, 0xCA, 0x9A, 0x3B, 0x00, 0x00, 0x00, 0x00];
        assert_eq!(encode(&some_u64).unwrap(), expected_some_u64);
        assert_eq!(decode::<Option<u64>>(&expected_some_u64).unwrap(), some_u64);
        assert_eq!(
            decode::<Option<u64>>(&encode(&some_u64).unwrap()).unwrap(),
            some_u64
        );

        // --- Option<String> ---
        let none_str: Option<String> = None;
        assert_eq!(encode(&none_str).unwrap(), [0x00]);
        let some_str: Option<String> = Some(String::from("test"));
        let expected_some_str = [
            0x01, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, b't', b'e', b's', b't',
        ];
        assert_eq!(encode(&some_str).unwrap(), expected_some_str);
        assert_eq!(
            decode::<Option<String>>(&expected_some_str).unwrap(),
            some_str
        );

        // --- Unit tuple () ---
        assert_eq!(encode(&()).unwrap(), Vec::<u8>::new());
        assert_eq!(decode::<()>(&[]).unwrap(), ());

        // --- Tuple (u32, String) ---
        let tuple_val = (42u32, String::from("KVNC"));
        let mut expected_tuple = vec![42, 0, 0, 0];
        expected_tuple.extend_from_slice(&[4, 0, 0, 0, 0, 0, 0, 0, b'K', b'V', b'N', b'C']);
        assert_eq!(encode(&tuple_val).unwrap(), expected_tuple);
        assert_eq!(decode::<(u32, String)>(&expected_tuple).unwrap(), tuple_val);
        assert_eq!(
            decode::<(u32, String)>(&encode(&tuple_val).unwrap()).unwrap(),
            tuple_val
        );

        // --- Triple ([u8; 4], bool, u64) ---
        let triple = ([1u8, 2, 3, 4], true, 100u64);
        let mut expected_triple = vec![1, 2, 3, 4, 1];
        expected_triple.extend_from_slice(&100u64.to_le_bytes());
        assert_eq!(encode(&triple).unwrap(), expected_triple);
        assert_eq!(
            decode::<([u8; 4], bool, u64)>(&expected_triple).unwrap(),
            triple
        );
    }

    #[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
    struct TestTokenMetadata {
        symbol: String,
        decimals: u8,
        total_supply: u128,
        is_paused: bool,
    }

    #[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
    struct TestTransferEvent {
        from: [u8; 32],
        to: [u8; 32],
        amount: u128,
        memo: Option<String>,
    }

    #[test]
    fn test_abi_custom_structs_canonical_vectors_and_roundtrip() {
        let meta = TestTokenMetadata {
            symbol: String::from("KVNC"),
            decimals: 9,
            total_supply: 2_500_000_000,
            is_paused: false,
        };
        let mut expected_meta = vec![4, 0, 0, 0, 0, 0, 0, 0, b'K', b'V', b'N', b'C'];
        expected_meta.push(9);
        expected_meta.extend_from_slice(&2_500_000_000u128.to_le_bytes());
        expected_meta.push(0); // false
        assert_eq!(encode(&meta).unwrap(), expected_meta);
        assert_eq!(decode::<TestTokenMetadata>(&expected_meta).unwrap(), meta);
        assert_eq!(
            decode::<TestTokenMetadata>(&encode(&meta).unwrap()).unwrap(),
            meta
        );

        let transfer = TestTransferEvent {
            from: [0x11; 32],
            to: [0x22; 32],
            amount: 50_000_000_000,
            memo: Some(String::from("mining reward")),
        };
        let mut expected_transfer = Vec::new();
        expected_transfer.extend_from_slice(&[0x11; 32]);
        expected_transfer.extend_from_slice(&[0x22; 32]);
        expected_transfer.extend_from_slice(&50_000_000_000u128.to_le_bytes());
        expected_transfer.push(1); // Some
        let memo_bytes = b"mining reward";
        expected_transfer.extend_from_slice(&(memo_bytes.len() as u64).to_le_bytes());
        expected_transfer.extend_from_slice(memo_bytes);

        assert_eq!(encode(&transfer).unwrap(), expected_transfer);
        assert_eq!(
            decode::<TestTransferEvent>(&expected_transfer).unwrap(),
            transfer
        );
        assert_eq!(
            decode::<TestTransferEvent>(&encode(&transfer).unwrap()).unwrap(),
            transfer
        );
    }

    #[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
    enum TestEnum {
        Alpha,
        Beta,
        Tuple(u64, bool),
        Struct { id: u32, label: String },
    }

    #[test]
    fn test_abi_enums_canonical_vectors_and_roundtrip() {
        // Unit variant 0: Alpha
        let alpha = TestEnum::Alpha;
        let expected_alpha = [0x00, 0x00, 0x00, 0x00];
        assert_eq!(encode(&alpha).unwrap(), expected_alpha);
        assert_eq!(decode::<TestEnum>(&expected_alpha).unwrap(), alpha);

        // Unit variant 1: Beta
        let beta = TestEnum::Beta;
        let expected_beta = [0x01, 0x00, 0x00, 0x00];
        assert_eq!(encode(&beta).unwrap(), expected_beta);
        assert_eq!(decode::<TestEnum>(&expected_beta).unwrap(), beta);

        // Tuple variant 2: Tuple(1000, true)
        let tuple_var = TestEnum::Tuple(1000, true);
        let mut expected_tuple_var = vec![0x02, 0x00, 0x00, 0x00];
        expected_tuple_var.extend_from_slice(&1000u64.to_le_bytes());
        expected_tuple_var.push(1); // true
        assert_eq!(encode(&tuple_var).unwrap(), expected_tuple_var);
        assert_eq!(decode::<TestEnum>(&expected_tuple_var).unwrap(), tuple_var);
        assert_eq!(
            decode::<TestEnum>(&encode(&tuple_var).unwrap()).unwrap(),
            tuple_var
        );

        // Struct variant 3: Struct { id: 42, label: "kovanica" }
        let struct_var = TestEnum::Struct {
            id: 42,
            label: String::from("kovanica"),
        };
        let mut expected_struct_var = vec![0x03, 0x00, 0x00, 0x00];
        expected_struct_var.extend_from_slice(&42u32.to_le_bytes());
        expected_struct_var.extend_from_slice(&8u64.to_le_bytes());
        expected_struct_var.extend_from_slice(b"kovanica");
        assert_eq!(encode(&struct_var).unwrap(), expected_struct_var);
        assert_eq!(
            decode::<TestEnum>(&expected_struct_var).unwrap(),
            struct_var
        );
        assert_eq!(
            decode::<TestEnum>(&encode(&struct_var).unwrap()).unwrap(),
            struct_var
        );
    }

    #[test]
    fn test_error_code_encoding_across_all_variants() {
        let test_cases = [
            (ContractError::Unauthorized, -1, 0),
            (ContractError::InsufficientBalance, -2, 1),
            (ContractError::InvalidInput, -3, 2),
            (ContractError::AlreadyExists, -4, 3),
            (ContractError::NotFound, -5, 4),
            (ContractError::Expired, -6, 5),
            (ContractError::NotExpired, -7, 6),
            (ContractError::HashMismatch, -8, 7),
            (ContractError::ThresholdNotMet, -9, 8),
            (ContractError::Paused, -10, 9),
            (ContractError::Overflow, -11, 10),
            (ContractError::Custom(0), -12, 11),
            (ContractError::Custom(42), -12, 11),
            (ContractError::Custom(9999), -12, 11),
        ];

        for (err, expected_packed, expected_code) in test_cases {
            assert_eq!(
                err.code(),
                expected_code,
                "Error variant {err:?} must have numeric code {expected_code}"
            );
            assert_eq!(
                encode_error(&err),
                expected_packed,
                "Error variant {err:?} must pack to {expected_packed}"
            );
            assert!(
                (-12..=-1).contains(&expected_packed),
                "Packed code {expected_packed} must fall in -12..=-1"
            );

            // Verify decode roundtrip from code
            let decoded_from_code =
                ContractError::from_code(expected_code).expect("must decode valid code");
            if let ContractError::Custom(_) = err {
                assert_eq!(decoded_from_code, ContractError::Custom(11));
            } else {
                assert_eq!(decoded_from_code, err);
            }

            // Verify packed word decode formula: code = -1 - packed
            let unpacked_code = -1 - expected_packed;
            assert_eq!(unpacked_code, expected_code);
        }

        // Verify invalid codes return None
        assert_eq!(ContractError::from_code(-1), None);
        assert_eq!(ContractError::from_code(12), None);
        assert_eq!(ContractError::from_code(100), None);
    }

    #[test]
    fn test_abi_malformed_decoding_rejections() {
        // Truncated u64
        assert!(matches!(
            decode::<u64>(&[0x01, 0x02]),
            Err(ContractError::InvalidInput)
        ));

        // Invalid bool (must be 0 or 1)
        assert!(matches!(
            decode::<bool>(&[0x02]),
            Err(ContractError::InvalidInput)
        ));

        // String with length claiming 100 bytes but payload truncated
        let truncated_str = [100, 0, 0, 0, 0, 0, 0, 0, b'a', b'b'];
        assert!(matches!(
            decode::<String>(&truncated_str),
            Err(ContractError::InvalidInput)
        ));

        // Invalid enum variant index (e.g. index 55)
        let invalid_enum_variant = [55, 0, 0, 0];
        assert!(matches!(
            decode::<TestEnum>(&invalid_enum_variant),
            Err(ContractError::InvalidInput)
        ));
    }
}
