use actix_web::{
    web::{self, Data, Path},
    HttpResponse,
};
use rustical_store::SubscriptionStore;

async fn handle_delete<S: SubscriptionStore + ?Sized>(
    store: Data<S>,
    path: Path<String>,
) -> Result<HttpResponse, rustical_store::Error> {
    let id = path.into_inner();
    store.delete_subscription(&id).await?;
    Ok(HttpResponse::NoContent().body("Unregistered"))
}

pub fn subscription_resource<S: SubscriptionStore + ?Sized>() -> actix_web::Resource {
    web::resource("/subscription/{id}")
        .name("subscription")
        .delete(handle_delete::<S>)
}
