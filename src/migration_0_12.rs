use ical::parser::{ical::IcalObjectParser, vcard::VcardParser};
use rustical_store::{AddressbookStore, CalendarStore, auth::AuthenticationProvider};
use tracing::{error, info};

pub async fn validate_calendar_objects_0_12(
    principal_store: &impl AuthenticationProvider,
    cal_store: &impl CalendarStore,
) -> Result<(), rustical_store::Error> {
    let mut success = true;
    for principal in principal_store.get_principals().await? {
        for calendar in cal_store.get_calendars(&principal.id).await? {
            for (object_id, object) in cal_store
                .get_objects(&calendar.principal, &calendar.id)
                .await?
            {
                if let Err(err) =
                    IcalObjectParser::from_slice(object.get_ics().as_bytes()).expect_one()
                {
                    success = false;
                    error!(
                        "An error occured parsing a calendar object: principal={principal}, calendar={calendar}, object_id={object_id}: {err}",
                        principal = principal.id,
                        calendar = calendar.id,
                    );
                    println!("{}", object.get_ics());
                }
            }
        }
    }
    if success {
        info!("Your calendar data seems to be valid in the next major version.");
    } else {
        error!(
            "Not all calendar objects will be successfully parsed in the next major version (v0.12).
This will not cause issues in this version, but please comment under the tracking issue on GitHub:
https://github.com/lennart-k/rustical/issues/165"
        );
    }
    Ok(())
}

pub async fn validate_address_objects_0_12(
    principal_store: &impl AuthenticationProvider,
    addr_store: &impl AddressbookStore,
) -> Result<(), rustical_store::Error> {
    let mut success = true;
    for principal in principal_store.get_principals().await? {
        for addressbook in addr_store.get_addressbooks(&principal.id).await? {
            for (object_id, object) in addr_store
                .get_objects(&addressbook.principal, &addressbook.id)
                .await?
            {
                if let Err(err) = VcardParser::from_slice(object.get_vcf().as_bytes()).expect_one()
                {
                    success = false;
                    error!(
                        "An error occured parsing an address object: principal={principal}, addressbook={addressbook}, object_id={object_id}: {err}",
                        principal = principal.id,
                        addressbook = addressbook.id,
                    );
                    println!("{}", object.get_vcf());
                }
            }
        }
    }
    if success {
        info!("Your addressbook data seems to be valid in the next major version.");
    } else {
        error!(
            "Not all address objects will be successfully parsed in the next major version (v0.12).
This will not cause issues in this version, but please comment under the tracking issue on GitHub:
https://github.com/lennart-k/rustical/issues/165"
        );
    }
    Ok(())
}
