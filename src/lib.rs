//! ocsp-rs provides de/serialization for ocsp request and response in asn.1 der

use simple_asn1::{self, ASN1Block};

/// OCSP request structure
///
///```rust
///let ocsp_req = "306e306c304530433041300906052b0e\
///03021a05000414694d18a9be42f78026\
///14d4844f23601478b788200414397be0\
///02a2f571fd80dceb52a17a7f8b632be7\
///5502086378e51d448ff46da223302130\
///1f06092b060105050730010204120410\
///1cfc8fa3f5e15ed760707bc46670559b";
///let ocsp_bin = hex::decode(ocsp_req).unwrap();
///let asn1 = simple_asn1::from_der(&ocsp_bin[..]).unwrap();
///```
/// above binary data has ths following structure:
///
/// <table>
///   <tr>
///     <th>ASN1 hex</th>
///     <th>ASN1 Scheme</th>
///   </tr>
///   <tr>
///     <td>
///       <pre>
/// 30 6E
/// | 30 6C
/// |   30 45
/// |     30 43
/// |       30 41
/// |         30 09
/// |           06 05 2B0E03021A  --- OID
/// |           05 00             --- NULL
/// |         04 14 694D18A9BE42F7802614D4844F23601478B78820  --- OCTET
/// |         04 14 397BE002A2F571FD80DCEB52A17A7F8B632BE755  --- OCTET
/// |         02 08 6378E51D448FF46D  --- INT
/// |   A2 23  --- EXPLICIT TAG 2
/// |     30 21
/// |       30 1F
/// |         06 09 2B0601050507300102
/// |         04 12 04101CFC8FA3F5E15ED760707BC46670559B
/// |
/// |--- Sequence(30), 110 bytes(6E)
///       </pre>
///     </td>
///     <td>
///       <pre>
/// SEQUENCE {
///   SEQUENCE {
///   | SEQUENCE {
///   |   SEQUENCE {
///   |     SEQUENCE {
///   |     | SEQUENCE {
///   |     |    OBJECTIDENTIFIER 1.3.14.3.2.26 (id_sha1)
///   |     |    NULL
///   |     | }
///   |     | OCTETSTRING 694d18a9be42f7802614d4844f23601478b78820
///   |     | OCTETSTRING 397be002a2f571fd80dceb52a17a7f8b632be755
///   |     | INTEGER 0x6378e51d448ff46d
///   |     }
///   |   }
///   | }
///   | [2] {
///   |   SEQUENCE {
///   |   | SEQUENCE {
///   |   |   OBJECTIDENTIFIER 1.3.6.1.5.5.7.48.1.2
///   |   |   OCTETSTRING 04101cfc8fa3f5e15ed760707bc46670559b
///   |   | }
///   |   }
///   | }
///   }
/// }
///       </pre>
///     </td>
/// </table>
/// 
///```simple_asn1
/// [
/// | Sequence(0, [
/// | | Sequence(2, [
/// | | | Sequence(4, [
/// | | |   Sequence(6, [
/// | | |   | Sequence(8, [
/// | | |   |   Sequence(10, [
/// | | |   |   | ObjectIdentifier(12, OID([BigUint { data: [1] }, BigUint { data: [3] }, BigUint { data: [14] }, BigUint { data: [3] }, BigUint { data: [2] }, BigUint { data: [26] }])), 
/// | | |   |   | Null(19)
/// | | |   |   ]), 
/// | | |   |   OctetString(21, [105, 77, 24, 169, 190, 66, 247, 128, 38, 20, 212, 132, 79, 35, 96, 20, 120, 183, 136, 32]),
/// | | |   |   OctetString(43, [57, 123, 224, 2, 162, 245, 113, 253, 128, 220, 235, 82, 161, 122, 127, 139, 99, 43, 231, 85]), 
/// | | |   |   Integer(65, BigInt { sign: Plus, data: BigUint { data: [7167730720827241581] } })
/// | | |   | ])
/// | | |   ])
/// | | | ]), 
/// | | | Explicit(ContextSpecific, 75, BigUint { data: [2] }, Sequence(77, [
/// | | |   Sequence(79, [
/// | | |       ObjectIdentifier(81, OID([BigUint { data: [1] }, BigUint { data: [3] }, BigUint { data: [6] }, BigUint { data: [1] }, BigUint { data: [5] }, BigUint { data: [5] }, BigUint { data: [7] }, BigUint { data: [48] }, BigUint { data: [1] }, BigUint { data: [2] }])), 
/// | | |       OctetString(92, [4, 16, 28, 252, 143, 163, 245, 225, 94, 215, 96, 112, 123, 196, 102, 112, 85, 155])
/// | | |   ])
/// | | | ]))
/// | | ])
/// | ])
/// ]
///```
/// 
pub struct OcspRequest {
    data: Vec<ASN1Block>,
}

#[cfg(test)]
mod tests {
    use hex;
    use simple_asn1;
    /// test data produces an ocsp request generated by openssl.
    #[test]
    //#[ignore]
    fn ocsp_req_from_der() {
        let ocsp_req_hex = "306e306c304530433041300906052b0e\
03021a05000414694d18a9be42f78026\
14d4844f23601478b788200414397be0\
02a2f571fd80dceb52a17a7f8b632be7\
5502086378e51d448ff46da223302130\
1f06092b060105050730010204120410\
1cfc8fa3f5e15ed760707bc46670559b";
        let ocsp_req_bin = hex::decode(ocsp_req_hex).unwrap();
        let asn1b = simple_asn1::from_der(&ocsp_req_bin[..]).unwrap();
        println!("{:?}", asn1b);
    }
}
