use super::FilterElement;
use rstest::rstest;
use rustical_ical::AddressObject;
use rustical_xml::XmlDocument;

const VCF_1: &str = r"BEGIN:VCARD
VERSION:4.0
FN:Simon Perreault
N:Perreault;Simon;;;ing. jr,M.Sc.
BDAY:--0203
GENDER:M
EMAIL;TYPE=work:simon.perreault@viagenie.ca
END:VCARD";

const VCF_2: &str = r"BEGIN:VCARD
VERSION:4.0
N:Gump;Forrest;;Mr.;
FN:Forrest Gump
ORG:Bubba Gump Shrimp Co.
TITLE:Shrimp Man
PHOTO;MEDIATYPE=image/gif:http://www.example.com/dir_photos/my_photo.gif
TEL;TYPE=work,voice;VALUE=uri:tel:+1-111-555-1212
TEL;TYPE=home,voice;VALUE=uri:tel:+1-404-555-1212
ADR;TYPE=WORK;PREF=1;LABEL=100 Waters Edge\\nBaytown\\, LA 30314\\nUnited S
 tates of America:;;100 Waters Edge;Baytown;LA;30314;United States of Ameri
 ca
ADR;TYPE=HOME;LABEL=42 Plantation St.\\nBaytown\\, LA 30314\\nUnited States
  of America:;;42 Plantation St.;Baytown;LA;30314;United States of America
EMAIL:forrestgump@example.com
REV:20080424T195243Z
x-qq:21588891
UID:890a9da4-bb6d-4afb-9f32-b5eff6494a53
END:VCARD
";

const FILTER_1: &str = r#"
<?xml version="1.0" encoding="utf-8" ?>
<C:filter xmlns:C="urn:ietf:params:xml:ns:carddav">
    <C:prop-filter name="EMAIL" test="allof">
        <C:text-match collation="i;ascii-casemap">simon.perreault@viagenie.ca</C:text-match>
        <C:param-filter name="TYPE">
            <C:text-match match-type="equals" collation="i;unicode-casemap">WORK</C:text-match>
        </C:param-filter>
    </C:prop-filter>
</C:filter>
"#;

const FILTER_2: &str = r#"
<?xml version="1.0" encoding="utf-8" ?>
<C:filter xmlns:C="urn:ietf:params:xml:ns:carddav">
    <C:prop-filter name="EMAIL" test="anyof">
        <C:text-match collation="i;ascii-casemap">forrestgump@example.com</C:text-match>
        <C:param-filter name="TYPE">
            <C:text-match match-type="equals" collation="i;ascii-casemap">WORK</C:text-match>
        </C:param-filter>
    </C:prop-filter>
</C:filter>
"#;

#[rstest]
#[case(VCF_1, FILTER_1, true)]
#[case(VCF_2, FILTER_1, false)]
#[case(VCF_1, FILTER_2, true)]
#[case(VCF_2, FILTER_2, true)]
fn test_filter(#[case] vcf: &str, #[case] filter: &str, #[case] matches: bool) {
    dbg!(vcf);
    let obj = AddressObject::from_vcf(vcf.to_owned()).unwrap();
    let filter = FilterElement::parse_str(filter).unwrap();
    assert_eq!(matches, filter.matches(&obj));
}
