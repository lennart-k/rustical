use crate::Error;
mod elements;
mod prop_filter;
pub use elements::*;
#[allow(unused_imports)]
pub use prop_filter::{PropFilterElement, PropFilterable};
use rustical_ical::AddressObject;
use rustical_store::AddressbookStore;

#[cfg(test)]
mod tests;

pub async fn get_objects_addressbook_query<AS: AddressbookStore>(
    addr_query: &AddressbookQueryRequest,
    principal: &str,
    addressbook_id: &str,
    store: &AS,
) -> Result<Vec<AddressObject>, Error> {
    let mut objects = store.get_objects(principal, addressbook_id).await?;
    objects.retain(|object| addr_query.filter.matches(object));
    Ok(objects)
}
