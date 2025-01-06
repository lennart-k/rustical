use std::{collections::HashMap, io::BufReader};

use crate::{calendar::CalDateTime, Error};
use ical::parser::{
    vcard::{self, component::VcardContact},
    Component,
};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone)]
pub struct AddressObject {
    id: String,
    vcf: String,
    vcard: VcardContact,
}

impl AddressObject {
    pub fn from_vcf(object_id: String, vcf: String) -> Result<Self, Error> {
        let mut parser = vcard::VcardParser::new(BufReader::new(vcf.as_bytes()));
        let vcard = parser.next().ok_or(Error::NotFound)??;
        if parser.next().is_some() {
            return Err(Error::InvalidData(
                "multiple vcards, only one allowed".to_owned(),
            ));
        }
        Ok(Self {
            id: object_id,
            vcf,
            vcard,
        })
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

    pub fn get_birthday(&self) -> Option<CalDateTime> {
        let prop = self.vcard.get_property("BDAY")?;
        CalDateTime::parse_prop(prop, &HashMap::default()).unwrap_or(None)
    }
}
