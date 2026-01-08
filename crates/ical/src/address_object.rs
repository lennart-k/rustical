use crate::{CalendarObject, Error};
use ical::generator::Emitter;
use ical::parser::vcard::{self, component::VcardContact};
use sha2::{Digest, Sha256};
use std::{collections::HashMap, io::BufReader};

#[derive(Debug, Clone)]
pub struct AddressObject {
    vcf: String,
    vcard: VcardContact,
}

impl From<VcardContact> for AddressObject {
    fn from(vcard: VcardContact) -> Self {
        let vcf = vcard.generate();
        Self { vcf, vcard }
    }
}

impl AddressObject {
    pub fn from_vcf(vcf: String) -> Result<Self, Error> {
        let parser = vcard::VcardParser::new(BufReader::new(vcf.as_bytes()));
        let vcard = parser.expect_one()?;
        Ok(Self { vcf, vcard })
    }

    #[must_use]
    pub fn get_etag(&self) -> String {
        let mut hasher = Sha256::new();
        hasher.update(self.get_vcf());
        format!("\"{:x}\"", hasher.finalize())
    }

    #[must_use]
    pub fn get_vcf(&self) -> &str {
        &self.vcf
    }

    pub fn get_anniversary_object(&self) -> Result<Option<CalendarObject>, Error> {
        todo!();
    }

    pub fn get_birthday_object(&self) -> Result<Option<CalendarObject>, Error> {
        todo!();
    }

    /// Get significant dates associated with this address object
    pub fn get_significant_dates(&self) -> Result<HashMap<&'static str, CalendarObject>, Error> {
        let mut out = HashMap::new();
        if let Some(birthday) = self.get_birthday_object()? {
            out.insert("birthday", birthday);
        }
        if let Some(anniversary) = self.get_anniversary_object()? {
            out.insert("anniversary", anniversary);
        }
        Ok(out)
    }

    #[must_use]
    pub const fn get_vcard(&self) -> &VcardContact {
        &self.vcard
    }
}
