//! OCSP request

use asn1_der::{typed::Sequence, DerObject};
use futures::future::{BoxFuture, FutureExt};

use crate::err::{OcspError, Result};
use crate::{
    common::{
        OcspExt, TryIntoSequence, ASN1_EXPLICIT_0, ASN1_EXPLICIT_1, ASN1_EXPLICIT_2,
        ASN1_IA5STRING, ASN1_INTEGER, ASN1_NULL, ASN1_OCTET, ASN1_OID, ASN1_SEQUENCE,
    },
    err_at,
};

use log::debug;

/// Oid represents a 0x06 OID type in ASN.1  
/// in OpenSSL ocsp request, OID is followed by NULL 0x05
/// REVIEW 0x05
#[derive(Debug)]
pub struct Oid {
    id: Vec<u8>,
    //null: Vec<u8>,
}

impl Oid {
    /// get oid from raw sequence
    pub async fn parse(oid: &[u8]) -> Result<Self> {
        debug!("Parsing OID {:02X?}", oid);
        let s = oid.try_into()?;

        if s.len() != 2 {
            return Err(OcspError::Asn1LengthError("OID", err_at!()));
        }

        let id = s.get(0).map_err(OcspError::Asn1DecodingError)?;
        let nil = s.get(1).map_err(OcspError::Asn1DecodingError)?;
        if id.tag() != ASN1_OID || nil.tag() != ASN1_NULL {
            return Err(OcspError::Asn1MismatchError("OID", err_at!()));
        }

        Ok(Oid {
            id: id.value().to_vec(),
        })
    }
}

/// RFC 6960 CertID
pub struct CertId {
    hash_algo: Oid,
    issuer_name_hash: Vec<u8>,
    issuer_key_hash: Vec<u8>,
    serial_num: Vec<u8>,
}

impl CertId {
    /// get certid from raw bytes
    pub async fn parse(certid: &[u8]) -> Result<Self> {
        debug!("Parsing CERTID {:02X?}", certid);
        let s = certid.try_into()?;

        if s.len() != 4 {
            return Err(OcspError::Asn1LengthError("CertID", err_at!()));
        }

        let oid = s.get(0).map_err(OcspError::Asn1DecodingError)?;
        let name_hash = s.get(1).map_err(OcspError::Asn1DecodingError)?;
        let key_hash = s.get(2).map_err(OcspError::Asn1DecodingError)?;
        let sn = s.get(3).map_err(OcspError::Asn1DecodingError)?;

        if oid.tag() != ASN1_SEQUENCE
            || name_hash.tag() != ASN1_OCTET
            || key_hash.tag() != ASN1_OCTET
            || sn.tag() != ASN1_INTEGER
        {
            return Err(OcspError::Asn1MismatchError("CertId", err_at!()));
        }

        let oid = Oid::parse(oid.raw()).await?;
        let name_hash = name_hash.value().to_vec();
        let key_hash = key_hash.value().to_vec();
        let sn = sn.value().to_vec();

        Ok(CertId {
            hash_algo: oid,
            issuer_name_hash: name_hash,
            issuer_key_hash: key_hash,
            serial_num: sn,
        })
    }
}
/// RFC 6960 Request
pub struct OneReq {
    one_req: CertId,
    one_req_ext: Option<Vec<OcspExt>>,
}

impl OneReq {
    /// get single request
    pub async fn parse(onereq: Vec<u8>) -> Result<Self> {
        debug!("Parsing ONEREQ {:02X?}", onereq);
        let s = onereq.try_into()?;

        let certid = s.get(0).map_err(OcspError::Asn1DecodingError)?;
        let certid = CertId::parse(certid.raw()).await?;
        let mut ext = None;
        match s.len() {
            1 => {}
            2 => {
                let raw_ext = s.get_as(1).map_err(OcspError::Asn1DecodingError)?;
                ext = Some(OcspExt::parse(raw_ext).await?);
            }
            _ => {
                return Err(OcspError::Asn1LengthError("OneReq", err_at!()));
            }
        }

        Ok(OneReq {
            one_req: certid,
            one_req_ext: ext,
        })
    }
}

/// RFC 6960 TBSRequest  
/// version is omitted as data produced from OpenSSL doesn't contain version  
/// REVIEW: omit version in tbs request
pub struct TBSRequest {
    // explicit tag 0
    // version: u8,
    /// requestorName is OPTIONAL and indicates the name of the OCSP requestor.
    /// explicit 1
    requestor_name: Option<Vec<u8>>,
    /// requestList contains one or more single certificate status requests.
    request_list: Vec<OneReq>,
    /// requestExtensions is OPTIONAL and includes extensions applicable
    /// to the requests found in reqCert.
    request_ext: Option<Vec<OcspExt>>,
}

impl TBSRequest {
    /// parse a tbs request
    pub async fn parse(tbs: Vec<u8>) -> Result<Self> {
        debug!("Parsing TBSREQUEST {:02X?}", tbs);
        let mut name = None;
        let mut ext = None;
        let mut req: Vec<OneReq> = Vec::new();
        let s = tbs.try_into()?;

        for i in 0..s.len() {
            let tbs_item = s.get(i).map_err(OcspError::Asn1DecodingError)?;
            match tbs_item.tag() {
                ASN1_EXPLICIT_0 => {
                    unimplemented!()
                }
                ASN1_EXPLICIT_1 => {
                    let val = tbs_item.value();
                    let val = DerObject::decode(val).map_err(OcspError::Asn1DecodingError)?;
                    if val.tag() != ASN1_IA5STRING {
                        return Err(OcspError::Asn1MismatchError(
                            "TBS requestor name",
                            err_at!(),
                        ));
                    }
                    name = Some(val.value().to_vec());
                }
                ASN1_EXPLICIT_2 => {
                    let ext_list = tbs_item.value().to_vec();
                    let ext_list = OcspExt::parse(ext_list).await?;
                    ext = Some(ext_list);
                }
                ASN1_SEQUENCE => {
                    let req_list = tbs_item.try_into()?;
                    for j in 0..req_list.len() {
                        let onereq = req_list.get(j).map_err(OcspError::Asn1DecodingError)?;
                        let onereq = OneReq::parse(onereq.raw().to_vec()).await?;
                        req.push(onereq);
                    }
                }
                _ => {
                    return Err(OcspError::Asn1MismatchError("TBS Request", err_at!()));
                }
            }
        }
        Ok(TBSRequest {
            requestor_name: name,
            request_list: req,
            request_ext: ext,
        })
    }
}

/// optional signature in ocsp request
pub struct Signature {
    signing_algo: Oid,
    /// tho RFC 6960 indicates signature is BIT STRING,  
    /// which has arbitrary length comparing to OCTET,  
    /// but all signature length are multiple of 8,  
    /// so using Vec\<u8\> here.
    signature: Vec<u8>,
    /// [0] EXPLICIT SEQUENCE OF Certificate OPTIONAL
    certs: Option<Vec<Vec<u8>>>,
}

impl Signature {
    /// parsing ocsp signature from raw bytes
    pub async fn parse(sig: Vec<u8>) -> Result<Self> {
        debug!("Parsing SIGNATURE: {:02X?}", sig);
        let s = sig.try_into()?;

        let oid;
        let signature;

        //let cert = None;
        match s.len() {
            2 => {
                let id = s.get(0).map_err(OcspError::Asn1DecodingError)?;
                oid = Oid::parse(id.raw()).await?;

                signature = s
                    .get(1)
                    .map_err(OcspError::Asn1DecodingError)?
                    .raw()
                    .to_vec();
            }
            3 => {
                unimplemented!()
            }
            _ => return Err(OcspError::Asn1LengthError("SIGNATURE", err_at!())),
        }

        Ok(Signature {
            signing_algo: oid,
            signature: signature,
            // unimplemented 3
            certs: None,
        })
    }
}

/// RFC 6960 OCSPRequest
pub struct OcspRequest {
    /// RFC 6960 TBSRequest
    tbs_request: TBSRequest,
    /// RFC 6960 optionalSignature, explicit tag 0
    optional_signature: Option<Signature>,
}

impl OcspRequest {
    /// parse an ocsp request from raw bytes
    pub async fn parse(ocsp_req: Vec<u8>) -> Result<Self> {
        debug!("Parsing OCSP REQUEST: {:02X?}", ocsp_req);
        let s = ocsp_req.try_into()?;

        let req;
        let mut sig = None;
        match s.len() {
            1 => {}
            2 => {
                let sig_v8 = s.get(1).map_err(OcspError::Asn1DecodingError)?;
                match sig_v8.tag() {
                    ASN1_EXPLICIT_0 => {
                        let val = sig_v8.value();
                        let val = DerObject::decode(val).map_err(OcspError::Asn1DecodingError)?;
                        sig = Some(Signature::parse(val.value().to_vec()).await?);
                    }
                    _ => return Err(OcspError::Asn1MismatchError("SIGNATURE EXP 0", err_at!())),
                }
            }
            _ => {}
        }
        let req_v8 = s.get(0).map_err(OcspError::Asn1DecodingError)?;
        req = TBSRequest::parse(req_v8.raw().to_vec()).await?;

        Ok(OcspRequest {
            tbs_request: req,
            optional_signature: sig,
        })
    }
}

#[cfg(test)]
mod test {
    use asn1_der::{
        typed::{DerDecodable, Sequence},
        DerObject,
    };
    use env_logger;
    use hex;

    use super::{CertId, OcspRequest, Oid, OneReq, TBSRequest};

    // init log
    #[allow(dead_code)]
    fn init() {
        let _ = env_logger::builder()
            .is_test(true)
            .filter_level(log::LevelFilter::Debug)
            .try_init();
    }

    #[tokio::test]
    async fn ocsprequest_parse_from_v8() {
        let ocsp_req_hex = "306e306c304530433041300906052b0e\
    03021a05000414694d18a9be42f78026\
    14d4844f23601478b788200414397be0\
    02a2f571fd80dceb52a17a7f8b632be7\
    5502086378e51d448ff46da223302130\
    1f06092b060105050730010204120410\
    1cfc8fa3f5e15ed760707bc46670559b";
        let ocsp_req_v8 = hex::decode(ocsp_req_hex).unwrap();
        let ocsp_request = OcspRequest::parse(ocsp_req_v8).await;
        assert!(ocsp_request.is_ok());
    }

    // test confirms context specific tag cannot be recognized
    #[test]
    #[should_panic]
    fn context_specific_sequence() {
        let ocsp_req_hex = "306e306c304530433041300906052b0e\
    03021a05000414694d18a9be42f78026\
    14d4844f23601478b788200414397be0\
    02a2f571fd80dceb52a17a7f8b632be7\
    5502086378e51d448ff46da223302130\
    1f06092b060105050730010204120410\
    1cfc8fa3f5e15ed760707bc46670559b";
        let ocsp_req = hex::decode(ocsp_req_hex).unwrap();
        let der = DerObject::decode(&ocsp_req[..]).unwrap();
        //println!("tag {:02X?}\nvalue {:02X?}", der.header(), der.value());

        let tbs = DerObject::decode(der.value()).unwrap();
        //println!("tag {:02X?}\nvalue {:02X?}", tbs.header(), tbs.value());

        let _reqlist = DerObject::decode(tbs.value()).unwrap();
        //println!(
        //    "tag {:02X?}\nvalue {:02X?}",
        //    reqlist.header(),
        //    reqlist.value()
        //);

        let ocspseq = Sequence::decode(der.value()).unwrap();
        let t = ocspseq.get(1).unwrap().header();
        let v = ocspseq.get(1).unwrap().value();
        let mut t = t.to_vec();
        t.extend(v);
        //println!("context specific exp tag 2{:02X?}", t);
        let _ = Sequence::decode(&t[..]).unwrap();
    }

    // get one tbs request with nonce ext
    #[tokio::test]
    async fn parse_tbs_nonce_ext() {
        let tbs_hex = "306c304530433041300906052b0e\
    03021a05000414694d18a9be42f78026\
    14d4844f23601478b788200414397be0\
    02a2f571fd80dceb52a17a7f8b632be7\
    5502086378e51d448ff46da223302130\
    1f06092b060105050730010204120410\
    1cfc8fa3f5e15ed760707bc46670559b";
        let tbs_v8 = hex::decode(tbs_hex).unwrap();
        let _ = TBSRequest::parse(tbs_v8).await.unwrap();
    }

    // get one request with no extension on either request
    #[tokio::test]
    async fn parse_onereq_no_ext() {
        let onereq_hex = "30433041300906052b0e\
    03021a05000414694d18a9be42f78026\
    14d4844f23601478b788200414397be0\
    02a2f571fd80dceb52a17a7f8b632be7\
    5502086378e51d448ff46d";
        let onereq_v8 = hex::decode(onereq_hex).unwrap();
        let _ = OneReq::parse(onereq_v8).await.unwrap();
    }

    /// get certid from raw hex
    #[tokio::test]
    async fn parse_certid_v8() {
        let certid_hex = "3041300906052b0e\
    03021a05000414694d18a9be42f78026\
    14d4844f23601478b788200414397be0\
    02a2f571fd80dceb52a17a7f8b632be7\
    5502086378e51d448ff46d";
        let certid_v8 = hex::decode(certid_hex).unwrap();
        let _ = CertId::parse(&certid_v8[..]).await.unwrap();
    }

    // this proves asn1_der drops data after null tag in a sequence
    #[tokio::test]
    async fn parse_oid_null_drops() {
        let oid_hex = "300906052b0e03021a0500040107";
        let oid_v8 = hex::decode(oid_hex).unwrap();
        let _ = Oid::parse(&oid_v8[..]).await.unwrap();
        //let s = oid_v8.try_into().unwrap();
        //let d = s.get(1).unwrap();
        //println!("{:?}", d.header());
    }

    // get oid vec<u8> from raw hex
    #[tokio::test]
    async fn parse_oid_v8() {
        let oid_hex = "300906052b0e03021a0500";
        let oid_v8 = hex::decode(oid_hex).unwrap();
        let oid = Oid::parse(&oid_v8).await.unwrap();
        assert_eq!(oid.id, vec![0x2b, 0x0e, 0x03, 0x02, 0x1a]);
    }

    // display error with file & line info
    #[tokio::test]
    #[should_panic]
    async fn parse_oid_sequence_into_err() {
        let oid_hex = "300906052b0e03021a";
        let oid_v8 = hex::decode(oid_hex).unwrap();
        let _ = Oid::parse(&oid_v8[..]).await.unwrap();
    }

    // incorrect sequence length
    #[tokio::test]
    #[should_panic]
    async fn parse_oid_length_err() {
        let oid_hex = "3041300906052b0e\
    03021a05000414694d18a9be42f78026\
    14d4844f23601478b788200414397be0\
    02a2f571fd80dceb52a17a7f8b632be7\
    5502086378e51d448ff46d";
        let oid_v8 = hex::decode(oid_hex).unwrap();
        let _ = Oid::parse(&oid_v8[..]).await.unwrap();
    }

    // mismatch sequence
    #[tokio::test]
    #[should_panic]
    async fn parse_oid_mismatch_err() {
        let oid_hex = "300a06052b0e03021a0201ff";
        let oid_v8 = hex::decode(oid_hex).unwrap();
        let _ = Oid::parse(&oid_v8[..]).await.unwrap();
    }
}
