use std::future::Future;
use std::pin::Pin;
use std::str::from_utf8;
use ariadne::Cache;
use x509_parser::pem::Pem;
use x509_parser::x509::X509Version;
use crate::function::{Function, FunctionError, FunctionPackage};
use crate::value::Value;

pub mod convert;

pub fn package() -> FunctionPackage {
    let mut pkg = FunctionPackage::new();
    pkg.register("PEM".into(), PEM);
    pkg
}

#[derive(Debug)]
pub struct PEM;

impl Function for PEM {
    fn call<'v>(
        &'v self,
        input: &'v Value,
    ) -> Pin<Box<dyn Future<Output = Result<Value, FunctionError>> + 'v>> {
        Box::pin(async move {
            let mut bytes = Vec::new();

            if let Some(inner) = input.try_get_string() {
                bytes.extend_from_slice(inner.as_bytes());
            } else if let Some(inner) = input.try_get_octets() {
                bytes.extend_from_slice( &*inner);
            } else {
                return Err(FunctionError::Other("invalid input type".into()));
            };

            let mut certs: Vec<Value> = Vec::new();

            for pem in Pem::iter_from_buffer(&bytes) {
                let pem = pem.expect("Reading next PEM block failed");
                println!("---------------> {:?}", pem);
                if pem.label == "PUBLIC" {
                    println!("public key? {:?}", pem);
                } else {
                    if let Ok(x509) = pem.parse_x509() {
                        assert_eq!(x509.tbs_certificate.version, X509Version::V3);
                        let converted: Value = (&x509).into();
                        println!("LOOK HERE");
                        println!("{}", converted.display().await);
                        certs.push( converted );
                    }
                }
            }


            Ok(certs.into())
        } )
    }
}