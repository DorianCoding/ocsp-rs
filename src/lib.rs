//! ocsp-rs provides de/serialization for ocsp request and response in asn.1 der

pub mod asn1_req;
mod asn1_common;
pub mod err;
pub struct OcspRequest {}

#[cfg(test)]
mod tests {
    use asn1_der::{
        //Boolean, DerDecodable, DerEncodable, DerTypeView, Integer, Null, OctetString, Sequence,
        //SequenceVec, Utf8String
        typed::{DerDecodable, Sequence},
        DerObject, //SliceSink,
    };
    use hex;

    use super::asn1_req;
    /// test data produces an ocsp request generated by openssl.
    #[test]
    fn ocsp_req_from_der() {
        let ocsp_req_hex = "306e306c304530433041300906052b0e\
03021a05000414694d18a9be42f78026\
14d4844f23601478b788200414397be0\
02a2f571fd80dceb52a17a7f8b632be7\
5502086378e51d448ff46da223302130\
1f06092b060105050730010204120410\
1cfc8fa3f5e15ed760707bc46670559b";
        let ocsp_req_bin = hex::decode(ocsp_req_hex).unwrap();
        let asn1 = DerObject::decode(&ocsp_req_bin[..]).unwrap();
        let seq = Sequence::decode(asn1.raw()).unwrap();
        let first_item = seq.get(0).unwrap();
        let seq = Sequence::decode(first_item.raw()).unwrap();
        let _second_item = seq.get(1).unwrap();
    }

    #[test]
    fn ocsp_req_get_certid() {
        let ocsp_req_hex = "306e306c304530433041300906052b0e\
03021a05000414694d18a9be42f78026\
14d4844f23601478b788200414397be0\
02a2f571fd80dceb52a17a7f8b632be7\
5502086378e51d448ff46da223302130\
1f06092b060105050730010204120410\
1cfc8fa3f5e15ed760707bc46670559b";
        let ocsp_req_bin = hex::decode(ocsp_req_hex).unwrap();
        let asn1 = DerObject::decode(&ocsp_req_bin[..]).unwrap();
        let seq = Sequence::decode(asn1.raw()).unwrap();
        let asn1 = asn1_req::OcspRequestAsn1 { seq: seq };
        let mut res = Vec::new();
        let mut val: Vec<Vec<u8>> = Vec::new();
        let _ = asn1.extract_certid(&mut res, &mut val);
        println!("{:02X?} ++ {:02X?}", res, val);
    }
}
