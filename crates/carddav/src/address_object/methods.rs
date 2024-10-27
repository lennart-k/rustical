use super::resource::AddressObjectPathComponents;
use crate::Error;
use actix_web::http::header;
use actix_web::http::header::HeaderValue;
use actix_web::web::{Data, Path};
use actix_web::HttpRequest;
use actix_web::HttpResponse;
use rustical_store::auth::User;
use rustical_store::model::AddressObject;
use rustical_store::AddressbookStore;
use tokio::sync::RwLock;
use tracing::instrument;
use tracing_actix_web::RootSpan;

#[instrument(parent = root_span.id(), skip(store, root_span))]
pub async fn get_object<AS: AddressbookStore + ?Sized>(
    path: Path<AddressObjectPathComponents>,
    store: Data<RwLock<AS>>,
    user: User,
    root_span: RootSpan,
) -> Result<HttpResponse, Error> {
    let AddressObjectPathComponents {
        principal,
        cal_id,
        object_id,
    } = path.into_inner();

    if user.id != principal {
        return Ok(HttpResponse::Unauthorized().body(""));
    }

    let addressbook = store
        .read()
        .await
        .get_addressbook(&principal, &cal_id)
        .await?;
    if user.id != addressbook.principal {
        return Ok(HttpResponse::Unauthorized().body(""));
    }

    let object = store
        .read()
        .await
        .get_object(&principal, &cal_id, &object_id)
        .await?;

    Ok(HttpResponse::Ok()
        .insert_header(("ETag", object.get_etag()))
        .insert_header(("Content-Type", "text/vcard"))
        .body(object.get_vcf().to_owned()))
}

#[instrument(parent = root_span.id(), skip(store, req, root_span))]
pub async fn put_object<AS: AddressbookStore + ?Sized>(
    path: Path<AddressObjectPathComponents>,
    store: Data<RwLock<AS>>,
    body: String,
    user: User,
    req: HttpRequest,
    root_span: RootSpan,
) -> Result<HttpResponse, Error> {
    let AddressObjectPathComponents {
        principal,
        cal_id: addressbook_id,
        object_id,
    } = path.into_inner();

    if user.id != principal {
        return Ok(HttpResponse::Unauthorized().body(""));
    }

    let addressbook = store
        .read()
        .await
        .get_addressbook(&principal, &addressbook_id)
        .await?;
    if user.id != addressbook.principal {
        return Ok(HttpResponse::Unauthorized().body(""));
    }

    // TODO: implement If-Match
    //
    let mut store_write = store.write().await;

    if Some(&HeaderValue::from_static("*")) == req.headers().get(header::IF_NONE_MATCH) {
        // Only write if not existing
        match store_write
            .get_object(&principal, &addressbook_id, &object_id)
            .await
        {
            Ok(_) => {
                // Conflict
                return Ok(HttpResponse::Conflict().body("Resource with this URI already exists"));
            }
            Err(rustical_store::Error::NotFound) => {
                // Path unused, we can proceed
            }
            Err(err) => {
                // Some unknown error :(
                return Err(err.into());
            }
        }
    }

    let object = AddressObject::from_vcf(object_id, body)?;
    store_write
        .put_object(principal, addressbook_id, object)
        .await?;

    Ok(HttpResponse::Created().body(""))
}
