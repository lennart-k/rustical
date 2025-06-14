use super::ChangeOperation;
use async_trait::async_trait;
use derive_more::derive::Constructor;
use rustical_ical::AddressObject;
use rustical_store::{
    Addressbook, AddressbookStore, CollectionOperation, CollectionOperationInfo, Error,
    synctoken::format_synctoken,
};
use sqlx::{Acquire, Executor, Sqlite, SqlitePool, Transaction};
use tokio::sync::mpsc::Sender;
use tracing::{error, instrument};

#[derive(Debug, Clone)]
struct AddressObjectRow {
    id: String,
    vcf: String,
}

impl TryFrom<AddressObjectRow> for AddressObject {
    type Error = crate::Error;

    fn try_from(value: AddressObjectRow) -> Result<Self, Self::Error> {
        Ok(Self::from_vcf(value.id, value.vcf)?)
    }
}

#[derive(Debug, Constructor)]
pub struct SqliteAddressbookStore {
    db: SqlitePool,
    sender: Sender<CollectionOperation>,
}

impl SqliteAddressbookStore {
    async fn _get_addressbook<'e, E: Executor<'e, Database = Sqlite>>(
        executor: E,
        principal: &str,
        id: &str,
        show_deleted: bool,
    ) -> Result<Addressbook, rustical_store::Error> {
        let addressbook = sqlx::query_as!(
            Addressbook,
            r#"SELECT principal, id, synctoken, displayname, description, deleted_at, push_topic
                FROM addressbooks
                WHERE (principal, id) = (?, ?)
                AND ((deleted_at IS NULL) OR ?) "#,
            principal,
            id,
            show_deleted
        )
        .fetch_one(executor)
        .await
        .map_err(crate::Error::from)?;
        Ok(addressbook)
    }

    async fn _get_addressbooks<'e, E: Executor<'e, Database = Sqlite>>(
        executor: E,
        principal: &str,
    ) -> Result<Vec<Addressbook>, rustical_store::Error> {
        let addressbooks = sqlx::query_as!(
            Addressbook,
            r#"SELECT principal, id, synctoken, displayname, description, deleted_at, push_topic
                FROM addressbooks
                WHERE principal = ? AND deleted_at IS NULL"#,
            principal
        )
        .fetch_all(executor)
        .await
        .map_err(crate::Error::from)?;
        Ok(addressbooks)
    }

    async fn _get_deleted_addressbooks<'e, E: Executor<'e, Database = Sqlite>>(
        executor: E,
        principal: &str,
    ) -> Result<Vec<Addressbook>, rustical_store::Error> {
        let addressbooks = sqlx::query_as!(
            Addressbook,
            r#"SELECT principal, id, synctoken, displayname, description, deleted_at, push_topic
                FROM addressbooks
                WHERE principal = ? AND deleted_at IS NOT NULL"#,
            principal
        )
        .fetch_all(executor)
        .await
        .map_err(crate::Error::from)?;
        Ok(addressbooks)
    }

    async fn _update_addressbook<'e, E: Executor<'e, Database = Sqlite>>(
        executor: E,
        principal: String,
        id: String,
        addressbook: Addressbook,
    ) -> Result<(), rustical_store::Error> {
        let result = sqlx::query!(
            r#"UPDATE addressbooks SET principal = ?, id = ?, displayname = ?, description = ?, push_topic = ?
                WHERE (principal, id) = (?, ?)"#,
            addressbook.principal,
            addressbook.id,
            addressbook.displayname,
            addressbook.description,
            addressbook.push_topic,
            principal,
            id
        )
        .execute(executor)
        .await
        .map_err(crate::Error::from)?;
        if result.rows_affected() == 0 {
            return Err(rustical_store::Error::NotFound);
        }
        Ok(())
    }

    async fn _insert_addressbook<'e, E: Executor<'e, Database = Sqlite>>(
        executor: E,
        addressbook: Addressbook,
    ) -> Result<(), rustical_store::Error> {
        sqlx::query!(
            r#"INSERT INTO addressbooks (principal, id, displayname, description, push_topic)
                VALUES (?, ?, ?, ?, ?)"#,
            addressbook.principal,
            addressbook.id,
            addressbook.displayname,
            addressbook.description,
            addressbook.push_topic,
        )
        .execute(executor)
        .await
        .map_err(crate::Error::from)?;
        Ok(())
    }

    async fn _delete_addressbook<'e, E: Executor<'e, Database = Sqlite>>(
        executor: E,
        principal: &str,
        addressbook_id: &str,
        use_trashbin: bool,
    ) -> Result<(), rustical_store::Error> {
        match use_trashbin {
            true => {
                sqlx::query!(
                    r#"UPDATE addressbooks SET deleted_at = datetime() WHERE (principal, id) = (?, ?)"#,
                    principal, addressbook_id
                )
                .execute(executor)
                .await.map_err(crate::Error::from)?;
            }
            false => {
                sqlx::query!(
                    r#"DELETE FROM addressbooks WHERE (principal, id) = (?, ?)"#,
                    principal,
                    addressbook_id
                )
                .execute(executor)
                .await
                .map_err(crate::Error::from)?;
            }
        };

        Ok(())
    }

    async fn _restore_addressbook<'e, E: Executor<'e, Database = Sqlite>>(
        executor: E,
        principal: &str,
        addressbook_id: &str,
    ) -> Result<(), rustical_store::Error> {
        sqlx::query!(
            r"UPDATE addressbooks SET deleted_at = NULL WHERE (principal, id) = (?, ?)",
            principal,
            addressbook_id
        )
        .execute(executor)
        .await
        .map_err(crate::Error::from)?;
        Ok(())
    }

    async fn _sync_changes<'a, A: Acquire<'a, Database = Sqlite>>(
        acquire: A,
        principal: &str,
        addressbook_id: &str,
        synctoken: i64,
    ) -> Result<(Vec<AddressObject>, Vec<String>, i64), rustical_store::Error> {
        struct Row {
            object_id: String,
            synctoken: i64,
        }

        let mut conn = acquire.acquire().await.map_err(crate::Error::from)?;

        let changes = sqlx::query_as!(
            Row,
            r#"
                SELECT DISTINCT object_id, max(0, synctoken) as "synctoken!: i64" from addressobjectchangelog
                WHERE synctoken > ?
                ORDER BY synctoken ASC
            "#,
            synctoken
        )
        .fetch_all(&mut *conn)
        .await.map_err(crate::Error::from)?;

        let mut objects = vec![];
        let mut deleted_objects = vec![];

        let new_synctoken = changes
            .last()
            .map(|&Row { synctoken, .. }| synctoken)
            .unwrap_or(0);

        for Row { object_id, .. } in changes {
            match Self::_get_object(&mut *conn, principal, addressbook_id, &object_id, false).await
            {
                Ok(object) => objects.push(object),
                Err(rustical_store::Error::NotFound) => deleted_objects.push(object_id),
                Err(err) => return Err(err),
            }
        }

        Ok((objects, deleted_objects, new_synctoken))
    }

    async fn _get_objects<'e, E: Executor<'e, Database = Sqlite>>(
        executor: E,
        principal: &str,
        addressbook_id: &str,
    ) -> Result<Vec<AddressObject>, rustical_store::Error> {
        sqlx::query_as!(
            AddressObjectRow,
            "SELECT id, vcf FROM addressobjects WHERE principal = ? AND addressbook_id = ? AND deleted_at IS NULL",
            principal,
            addressbook_id
        )
        .fetch_all(executor)
        .await.map_err(crate::Error::from)?
        .into_iter()
        .map(|row| row.try_into().map_err(rustical_store::Error::from))
        .collect()
    }

    async fn _get_object<'e, E: Executor<'e, Database = Sqlite>>(
        executor: E,
        principal: &str,
        addressbook_id: &str,
        object_id: &str,
        show_deleted: bool,
    ) -> Result<AddressObject, rustical_store::Error> {
        Ok(sqlx::query_as!(
            AddressObjectRow,
            "SELECT id, vcf FROM addressobjects WHERE (principal, addressbook_id, id) = (?, ?, ?) AND ((deleted_at IS NULL) or ?)",
            principal,
            addressbook_id,
            object_id,
            show_deleted
        )
        .fetch_one(executor)
        .await
        .map_err(crate::Error::from)?
        .try_into()?)
    }

    async fn _put_object<'e, E: Executor<'e, Database = Sqlite>>(
        executor: E,
        principal: String,
        addressbook_id: String,
        object: AddressObject,
        overwrite: bool,
    ) -> Result<(), rustical_store::Error> {
        let (object_id, vcf) = (object.get_id(), object.get_vcf());

        (if overwrite {
            sqlx::query!(
            "REPLACE INTO addressobjects (principal, addressbook_id, id, vcf) VALUES (?, ?, ?, ?)",
            principal,
            addressbook_id,
            object_id,
            vcf
        )
        } else {
            // If the object already exists a database error is thrown and handled in error.rs
            sqlx::query!(
            "INSERT INTO addressobjects (principal, addressbook_id, id, vcf) VALUES (?, ?, ?, ?)",
            principal,
            addressbook_id,
            object_id,
            vcf
        )
        })
        .execute(executor)
        .await
        .map_err(crate::Error::from)?;

        Ok(())
    }

    async fn _delete_object<'e, E: Executor<'e, Database = Sqlite>>(
        executor: E,
        principal: &str,
        addressbook_id: &str,
        object_id: &str,
        use_trashbin: bool,
    ) -> Result<(), rustical_store::Error> {
        match use_trashbin {
            true => {
                sqlx::query!(
                    "UPDATE addressobjects SET deleted_at = datetime(), updated_at = datetime() WHERE (principal, addressbook_id, id) = (?, ?, ?)",
                    principal,
                    addressbook_id,
                    object_id
                )
                .execute(executor)
                .await.map_err(crate::Error::from)?;
            }
            false => {
                sqlx::query!(
                    "DELETE FROM addressobjects WHERE addressbook_id = ? AND id = ?",
                    addressbook_id,
                    object_id
                )
                .execute(executor)
                .await
                .map_err(crate::Error::from)?;
            }
        };
        Ok(())
    }

    async fn _restore_object<'e, E: Executor<'e, Database = Sqlite>>(
        executor: E,
        principal: &str,
        addressbook_id: &str,
        object_id: &str,
    ) -> Result<(), rustical_store::Error> {
        sqlx::query!(
            r#"UPDATE addressobjects SET deleted_at = NULL, updated_at = datetime() WHERE (principal, addressbook_id, id) = (?, ?, ?)"#,
            principal,
            addressbook_id,
            object_id
        )
        .execute(executor)
        .await.map_err(crate::Error::from)?;
        Ok(())
    }
}

#[async_trait]
impl AddressbookStore for SqliteAddressbookStore {
    #[instrument]
    async fn get_addressbook(
        &self,
        principal: &str,
        id: &str,
        show_deleted: bool,
    ) -> Result<Addressbook, rustical_store::Error> {
        Self::_get_addressbook(&self.db, principal, id, show_deleted).await
    }

    #[instrument]
    async fn get_addressbooks(
        &self,
        principal: &str,
    ) -> Result<Vec<Addressbook>, rustical_store::Error> {
        Self::_get_addressbooks(&self.db, principal).await
    }

    #[instrument]
    async fn get_deleted_addressbooks(
        &self,
        principal: &str,
    ) -> Result<Vec<Addressbook>, rustical_store::Error> {
        Self::_get_deleted_addressbooks(&self.db, principal).await
    }

    #[instrument]
    async fn update_addressbook(
        &self,
        principal: String,
        id: String,
        addressbook: Addressbook,
    ) -> Result<(), rustical_store::Error> {
        Self::_update_addressbook(&self.db, principal, id, addressbook).await
    }

    #[instrument]
    async fn insert_addressbook(
        &self,
        addressbook: Addressbook,
    ) -> Result<(), rustical_store::Error> {
        Self::_insert_addressbook(&self.db, addressbook).await
    }

    #[instrument]
    async fn delete_addressbook(
        &self,
        principal: &str,
        addressbook_id: &str,
        use_trashbin: bool,
    ) -> Result<(), rustical_store::Error> {
        let mut tx = self.db.begin().await.map_err(crate::Error::from)?;

        let addressbook =
            match Self::_get_addressbook(&mut *tx, principal, addressbook_id, use_trashbin).await {
                Ok(addressbook) => Some(addressbook),
                Err(Error::NotFound) => None,
                Err(err) => return Err(err),
            };

        Self::_delete_addressbook(&mut *tx, principal, addressbook_id, use_trashbin).await?;
        tx.commit().await.map_err(crate::Error::from)?;

        if let Some(addressbook) = addressbook {
            if let Err(err) = self.sender.try_send(CollectionOperation {
                data: CollectionOperationInfo::Delete,
                topic: addressbook.push_topic,
            }) {
                error!("Push notification about deleted addressbook failed: {err}");
            };
        }

        Ok(())
    }

    #[instrument]
    async fn restore_addressbook(
        &self,
        principal: &str,
        addressbook_id: &str,
    ) -> Result<(), rustical_store::Error> {
        Self::_restore_addressbook(&self.db, principal, addressbook_id).await
    }

    #[instrument]
    async fn sync_changes(
        &self,
        principal: &str,
        addressbook_id: &str,
        synctoken: i64,
    ) -> Result<(Vec<AddressObject>, Vec<String>, i64), rustical_store::Error> {
        Self::_sync_changes(&self.db, principal, addressbook_id, synctoken).await
    }

    #[instrument]
    async fn get_objects(
        &self,
        principal: &str,
        addressbook_id: &str,
    ) -> Result<Vec<AddressObject>, rustical_store::Error> {
        Self::_get_objects(&self.db, principal, addressbook_id).await
    }

    #[instrument]
    async fn get_object(
        &self,
        principal: &str,
        addressbook_id: &str,
        object_id: &str,
        show_deleted: bool,
    ) -> Result<AddressObject, rustical_store::Error> {
        Self::_get_object(&self.db, principal, addressbook_id, object_id, show_deleted).await
    }

    #[instrument]
    async fn put_object(
        &self,
        principal: String,
        addressbook_id: String,
        object: AddressObject,
        overwrite: bool,
    ) -> Result<(), rustical_store::Error> {
        let mut tx = self.db.begin().await.map_err(crate::Error::from)?;

        let object_id = object.get_id().to_owned();

        Self::_put_object(
            &mut *tx,
            principal.to_owned(),
            addressbook_id.to_owned(),
            object,
            overwrite,
        )
        .await?;

        let sync_token = log_object_operation(
            &mut tx,
            &principal,
            &addressbook_id,
            &object_id,
            ChangeOperation::Add,
        )
        .await
        .map_err(crate::Error::from)?;

        tx.commit().await.map_err(crate::Error::from)?;

        if let Err(err) = self.sender.try_send(CollectionOperation {
            data: CollectionOperationInfo::Content { sync_token },
            topic: self
                .get_addressbook(&principal, &addressbook_id, false)
                .await?
                .push_topic,
        }) {
            error!("Push notification about deleted addressbook failed: {err}");
        };

        Ok(())
    }

    #[instrument]
    async fn delete_object(
        &self,
        principal: &str,
        addressbook_id: &str,
        object_id: &str,
        use_trashbin: bool,
    ) -> Result<(), rustical_store::Error> {
        let mut tx = self.db.begin().await.map_err(crate::Error::from)?;

        Self::_delete_object(&mut *tx, principal, addressbook_id, object_id, use_trashbin).await?;

        let sync_token = log_object_operation(
            &mut tx,
            principal,
            addressbook_id,
            object_id,
            ChangeOperation::Delete,
        )
        .await
        .map_err(crate::Error::from)?;

        tx.commit().await.map_err(crate::Error::from)?;

        if let Err(err) = self.sender.try_send(CollectionOperation {
            data: CollectionOperationInfo::Content { sync_token },
            topic: self
                .get_addressbook(principal, addressbook_id, false)
                .await?
                .push_topic,
        }) {
            error!("Push notification about deleted addressbook failed: {err}");
        };
        Ok(())
    }

    #[instrument]
    async fn restore_object(
        &self,
        principal: &str,
        addressbook_id: &str,
        object_id: &str,
    ) -> Result<(), rustical_store::Error> {
        let mut tx = self.db.begin().await.map_err(crate::Error::from)?;

        Self::_restore_object(&mut *tx, principal, addressbook_id, object_id).await?;

        let sync_token = log_object_operation(
            &mut tx,
            principal,
            addressbook_id,
            object_id,
            ChangeOperation::Add,
        )
        .await
        .map_err(crate::Error::from)?;
        tx.commit().await.map_err(crate::Error::from)?;

        if let Err(err) = self.sender.try_send(CollectionOperation {
            data: CollectionOperationInfo::Content { sync_token },
            topic: self
                .get_addressbook(principal, addressbook_id, false)
                .await?
                .push_topic,
        }) {
            error!("Push notification about deleted addressbook failed: {err}");
        };

        Ok(())
    }

    #[instrument(skip(objects))]
    async fn import_addressbook(
        &self,
        principal: String,
        addressbook: Addressbook,
        objects: Vec<AddressObject>,
    ) -> Result<(), Error> {
        let mut tx = self.db.begin().await.map_err(crate::Error::from)?;

        let addressbook_id = addressbook.id.clone();
        Self::_insert_addressbook(&mut *tx, addressbook).await?;

        for object in objects {
            Self::_put_object(
                &mut *tx,
                principal.clone(),
                addressbook_id.clone(),
                object,
                false,
            )
            .await?;
        }

        tx.commit().await.map_err(crate::Error::from)?;
        Ok(())
    }
}

// Logs an operation to an address object
async fn log_object_operation(
    tx: &mut Transaction<'_, Sqlite>,
    principal: &str,
    addressbook_id: &str,
    object_id: &str,
    operation: ChangeOperation,
) -> Result<String, sqlx::Error> {
    struct Synctoken {
        synctoken: i64,
    }
    let Synctoken { synctoken } = sqlx::query_as!(
        Synctoken,
        r#"
        UPDATE addressbooks
        SET synctoken = synctoken + 1
        WHERE (principal, id) = (?1, ?2)
        RETURNING synctoken"#,
        principal,
        addressbook_id
    )
    .fetch_one(&mut **tx)
    .await?;

    sqlx::query!(
        r#"
        INSERT INTO addressobjectchangelog (principal, addressbook_id, object_id, "operation", synctoken)
        VALUES (?1, ?2, ?3, ?4, (
            SELECT synctoken FROM addressbooks WHERE (principal, id) = (?1, ?2)
        ))"#,
        principal,
        addressbook_id,
        object_id,
        operation
    )
    .execute(&mut **tx)
    .await?;
    Ok(format_synctoken(synctoken))
}
