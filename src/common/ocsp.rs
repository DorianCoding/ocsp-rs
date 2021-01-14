//! common ocsp components

use asn1_der::DerObject;
use tracing::{debug, trace};

use crate::common::asn1::{
    TryIntoSequence, ASN1_EXPLICIT_0, ASN1_EXPLICIT_1, ASN1_EXPLICIT_2, ASN1_OID,
};
use crate::err_at;
use crate::{err::OcspError, oid::*};

/// RFC 6960 4.4 OCSP extensions
#[derive(Debug)]
pub enum OcspExt {
    /// 4.4.1
    Nonce {
        ///id-pkix-ocsp 2
        oid: ConstOid,
        /// nonce value
        nonce: Vec<u8>,
    },
    /// 4.4.2  
    /// REVIEW: untested
    CrlRef {
        /// id-pkix-ocsp 3
        oid: ConstOid,
        /// EXPLICIT 0 IA5String OPTIONAL
        url: Option<Vec<u8>>,
        /// EXPLICIT 1 INTEGER OPTIONAL
        num: Option<Vec<u8>>,
        /// EXPLICIT 2 GeneralizedTime OPTIONAL
        time: Option<Vec<u8>>,
    },
}

impl OcspExt {
    /// parse ocsp extension  
    /// raw is sequence of list extensions  
    /// remove explicit and implicit tags first
    pub async fn parse<'d>(raw: &[u8]) -> Result<Vec<Self>, OcspError> {
        trace!("Parsing EXTENSION list {:02X?}", raw);

        let mut r: Vec<OcspExt> = Vec::new();

        debug!("Converting EXT data into asn1 sequence");
        let list = raw.try_into()?;
        for i in 0..list.len() {
            //let ext: Sequence = list.get_as(i).map_err(OcspError::Asn1DecodingError)?;
            let ext = list.get(i).map_err(OcspError::Asn1DecodingError)?;
            r.push(OcspExt::parse_oneext(ext.raw()).await?);
        }

        debug!("good extensions");
        Ok(r)
    }

    /// pass in each sequence of extension, return OcspExt
    async fn parse_oneext<'d>(oneext: &[u8]) -> Result<Self, OcspError> {
        trace!("Parsing SINGLE EXTENSION {:02X?}", oneext);
        debug!("Converting EXT data into asn1 sequence");
        let oneext = oneext.try_into()?;

        let oid = oneext.get(0).map_err(OcspError::Asn1DecodingError)?;
        debug!("Checking OID tag");
        if oid.tag() != ASN1_OID {
            return Err(OcspError::Asn1MismatchError("OID", err_at!()));
        }
        let val = oid.value();
        // translate oid
        debug!("Resolving OID");
        let ext = match b2i_oid(val).await {
            None => return Err(OcspError::Asn1OidUnknown(err_at!())),
            Some(v) => v,
        };

        let r = match ext.id {
            OCSP_EXT_NONCE_ID => {
                debug!("Found NONCE extension");
                OcspExt::Nonce {
                    oid: ext,
                    nonce: oneext
                        .get(1)
                        .map_err(OcspError::Asn1DecodingError)?
                        .value()
                        .to_vec(),
                }
            }
            OCSP_EXT_CRLREF_ID => {
                debug!("Found CRLREF extension");
                let mut url = None;
                let mut num = None;
                let mut time = None;
                for i in 1..oneext.len() {
                    let tmp = oneext.get(i).map_err(OcspError::Asn1DecodingError)?;
                    let val = match tmp.tag() {
                        ASN1_EXPLICIT_0..=ASN1_EXPLICIT_2 => tmp.value(),
                        _ => return Err(OcspError::Asn1MismatchError("Ext CrlRef", err_at!())),
                    };
                    match tmp.tag() {
                        ASN1_EXPLICIT_0 => {
                            let val =
                                DerObject::decode(val).map_err(OcspError::Asn1DecodingError)?;
                            url = Some(val.value().to_vec());
                        }
                        ASN1_EXPLICIT_1 => {
                            let val =
                                DerObject::decode(val).map_err(OcspError::Asn1DecodingError)?;
                            num = Some(val.value().to_vec());
                        }
                        ASN1_EXPLICIT_2 => {
                            let val =
                                DerObject::decode(val).map_err(OcspError::Asn1DecodingError)?;
                            time = Some(val.value().to_vec());
                        }
                        _ => {
                            return Err(OcspError::Asn1MismatchError(
                                "Ext CrlRef EXP tag",
                                err_at!(),
                            ))
                        }
                    }
                }

                OcspExt::CrlRef {
                    oid: ext,
                    url: url,
                    num: num,
                    time: time,
                }
            }
            OCSP_EXT_RESP_TYPE_ID
            | OCSP_EXT_ARCHIVE_CUTOFF_ID
            | OCSP_EXT_CRL_REASON_ID
            | OCSP_EXT_INVALID_DATE_ID
            | OCSP_EXT_SERVICE_LOCATOR_ID
            | OCSP_EXT_PREF_SIG_ALGS_ID
            | OCSP_EXT_EXTENDED_REVOKE_ID => {
                unimplemented!()
            }
            _ => return Err(OcspError::OcspExtUnknown(err_at!())),
        };

        debug!("good single extension");
        Ok(r)
    }
}