use crate::{CalendarObject, Error};
use ical::generator::Emitter;
use ical::parser::{
    Component,
    vcard::{self, component::VcardContact},
};
use sha2::{Digest, Sha256};
use std::{collections::HashMap, io::BufReader};

#[derive(Debug, Clone)]
pub struct AddressObject {
    id: String,
    vcf: String,
    vcard: VcardContact,
}

impl TryFrom<VcardContact> for AddressObject {
    type Error = Error;

    fn try_from(vcard: VcardContact) -> Result<Self, Self::Error> {
        let uid = vcard
            .get_uid()
            .ok_or_else(|| Error::InvalidData("missing UID".to_owned()))?
            .to_owned();
        let vcf = vcard.generate();
        Ok(Self {
            vcf,
            vcard,
            id: uid,
        })
    }
}

impl AddressObject {
    pub fn from_vcf(id: String, vcf: String) -> Result<Self, Error> {
        let mut parser = vcard::VcardParser::new(BufReader::new(vcf.as_bytes()));
        let vcard = parser.next().ok_or(Error::MissingContact)??;
        if parser.next().is_some() {
            return Err(Error::InvalidData(
                "multiple vcards, only one allowed".to_owned(),
            ));
        }
        Ok(Self { id, vcf, vcard })
    }

    #[must_use]
    pub fn get_id(&self) -> &str {
        &self.id
    }

    #[must_use]
    pub fn get_etag(&self) -> String {
        let mut hasher = Sha256::new();
        hasher.update(self.get_id());
        hasher.update(self.get_vcf());
        format!("\"{:x}\"", hasher.finalize())
    }

    #[must_use]
    pub fn get_vcf(&self) -> &str {
        &self.vcf
    }

    #[must_use]
    pub fn get_full_name(&self) -> Option<&str> {
        let prop = self.vcard.get_property("FN")?;
        prop.value.as_deref()
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
