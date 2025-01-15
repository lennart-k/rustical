use actix_web::{
    web::{self, Data, Path},
    HttpResponse,
};
use rustical_dav::xml::multistatus::PropstatElement;
use rustical_store::SubscriptionStore;
use rustical_xml::{XmlRootTag, XmlSerialize};

use crate::calendar::resource::CalendarProp;

async fn handle_delete<S: SubscriptionStore>(
    store: Data<S>,
    path: Path<String>,
) -> Result<HttpResponse, rustical_store::Error> {
    let id = path.into_inner();
    store.delete_subscription(&id).await?;
    Ok(HttpResponse::NoContent().body("Unregistered"))
}

pub fn subscription_resource<S: SubscriptionStore>() -> actix_web::Resource {
    web::resource("/subscription/{id}")
        .name("subscription")
        .delete(handle_delete::<S>)
}

#[derive(XmlSerialize, XmlRootTag)]
#[xml(root = b"push-message", ns = "rustical_dav::namespace::NS_DAVPUSH")]
pub struct PushMessage {
    propstat: PropstatElement<CalendarProp>,
}
