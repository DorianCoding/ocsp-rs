//! error definitions
use thiserror::Error;
/// crate error enum
#[derive(Error, Debug)]
pub enum OcspError {
    /// raw data does not start with a sequence
    #[error("Expecting data start with Sequence")]
    Asn1UnexpectedType,
    /// cannot convert raw data into asn1_der::typed::*;
    #[error("Unable to decode ASN1 data, originated from asn1_der crate")]
    Asn1DecodingError(#[from] asn1_der::Asn1DerError),
    /// unexpected result from sequence matching fn
    #[error("Unable to extract data from asn1 due to traversal issue")]
    Asn1ExtractionUnknownError,
    /// extractor cannot find matching sequence  
    /// eg. OID sequence is not 0x06, 0x05
    #[error("Unable to extract desired sequence of {0}")]
    Asn1MismatchError(String),
    /// ocsp request contains unexpected data
    /// case 1: no sequence in request
    /// case 2: ocsp request is not {0x30} or {0x30, 0xA0}
    #[error("Ocsp request contains unexpected data")]
    Asn1MalformedTBSRequest,
    /// unable to parse vec\<u8\> to &str   
    /// eg. requestorName
    #[error("Unable to deserialize string from ocsp req/resp")]
    Asn1Utf8Error(#[from] std::str::Utf8Error),

    /// OID length is not 2, 0x06, 0x05
    #[error("Unable to deserialize OID")]
    Asn1OidLengthError,

    /// CertID length is not 4
    #[error("Unable to deserialize CertID")]
    Asn1CertidLengthError,
}
