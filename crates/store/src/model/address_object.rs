use sha2::{Digest, Sha256};

use crate::Error;

#[derive(Debug, Clone)]
pub struct AddressObject {
    id: String,
    vcf: String,
}

impl AddressObject {
    pub fn from_vcf(object_id: String, vcf: String) -> Result<Self, Error> {
        Ok(Self { id: object_id, vcf })
    }
    pub fn get_id(&self) -> &str {
        &self.id
    }
    pub fn get_etag(&self) -> String {
        let mut hasher = Sha256::new();
        hasher.update(&self.id);
        hasher.update(self.get_vcf());
        format!("{:x}", hasher.finalize())
    }

    pub fn get_vcf(&self) -> &str {
        &self.vcf
    }
}
