use super::SqliteStore;
use crate::Error;
use crate::{
    model::{AddressObject, Addressbook},
    AddressbookStore,
};
use async_trait::async_trait;
use serde::Serialize;
use sqlx::{Sqlite, Transaction};
use tracing::instrument;

#[derive(Debug, Clone)]
struct AddressObjectRow {
    id: String,
    vcf: String,
}

impl TryFrom<AddressObjectRow> for AddressObject {
    type Error = Error;

    fn try_from(value: AddressObjectRow) -> Result<Self, Error> {
        Self::from_vcf(value.id, value.vcf)
    }
}

#[derive(Debug, Clone, Serialize, sqlx::Type)]
#[serde(rename_all = "kebab-case")]
enum AddressbookChangeOperation {
    // There's no distinction between Add and Modify
    Add,
    Delete,
}

// Logs an operation to the events
async fn log_object_operation(
    tx: &mut Transaction<'_, Sqlite>,
    principal: &str,
    addressbook_id: &str,
    object_id: &str,
    operation: AddressbookChangeOperation,
) -> Result<(), Error> {
    sqlx::query!(
        r#"
        UPDATE addressbooks
        SET synctoken = synctoken + 1
        WHERE (principal, id) = (?1, ?2)"#,
        principal,
        addressbook_id
    )
    .execute(&mut **tx)
    .await?;

    sqlx::query!(
        r#"
        INSERT INTO addressobjectchangelog (principal, addressbook_id, object_id, operation, synctoken)
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
    Ok(())
}

#[async_trait]
impl AddressbookStore for SqliteStore {
    #[instrument]
    async fn get_addressbook(&self, principal: &str, id: &str) -> Result<Addressbook, Error> {
        let addressbook = sqlx::query_as!(
            Addressbook,
            r#"SELECT principal, id, synctoken, displayname, description, deleted_at
                FROM addressbooks
                WHERE (principal, id) = (?, ?)"#,
            principal,
            id
        )
        .fetch_one(&self.db)
        .await?;
        Ok(addressbook)
    }

    #[instrument]
    async fn get_addressbooks(&self, principal: &str) -> Result<Vec<Addressbook>, Error> {
        let addressbooks = sqlx::query_as!(
            Addressbook,
            r#"SELECT principal, id, synctoken, displayname, description, deleted_at
                FROM addressbooks
                WHERE principal = ? AND deleted_at IS NULL"#,
            principal
        )
        .fetch_all(&self.db)
        .await?;
        Ok(addressbooks)
    }

    #[instrument]
    async fn update_addressbook(
        &mut self,
        principal: String,
        id: String,
        addressbook: Addressbook,
    ) -> Result<(), Error> {
        let result = sqlx::query!(
            r#"UPDATE addressbooks SET principal = ?, id = ?, displayname = ?, description = ?
                WHERE (principal, id) = (?, ?)"#,
            addressbook.principal,
            addressbook.id,
            addressbook.displayname,
            addressbook.description,
            principal,
            id
        )
        .execute(&self.db)
        .await?;
        if result.rows_affected() == 0 {
            return Err(Error::NotFound);
        }
        Ok(())
    }

    #[instrument]
    async fn insert_addressbook(&mut self, addressbook: Addressbook) -> Result<(), Error> {
        sqlx::query!(
            r#"INSERT INTO addressbooks (principal, id, displayname, description)
                VALUES (?, ?, ?, ?)"#,
            addressbook.principal,
            addressbook.id,
            addressbook.displayname,
            addressbook.description,
        )
        .execute(&self.db)
        .await?;
        Ok(())
    }

    #[instrument]
    async fn delete_addressbook(
        &mut self,
        principal: &str,
        addressbook_id: &str,
        use_trashbin: bool,
    ) -> Result<(), Error> {
        match use_trashbin {
            true => {
                sqlx::query!(
                    r#"UPDATE addressbooks SET deleted_at = datetime() WHERE (principal, id) = (?, ?)"#,
                    principal, addressbook_id
                )
                .execute(&self.db)
                .await?;
            }
            false => {
                sqlx::query!(
                    r#"DELETE FROM addressbooks WHERE (principal, id) = (?, ?)"#,
                    principal,
                    addressbook_id
                )
                .execute(&self.db)
                .await?;
            }
        };
        Ok(())
    }

    #[instrument]
    async fn restore_addressbook(
        &mut self,
        principal: &str,
        addressbook_id: &str,
    ) -> Result<(), Error> {
        sqlx::query!(
            r"UPDATE addressbooks SET deleted_at = NULL WHERE (principal, id) = (?, ?)",
            principal,
            addressbook_id
        )
        .execute(&self.db)
        .await?;
        Ok(())
    }

    #[instrument]
    async fn sync_changes(
        &self,
        principal: &str,
        addressbook_id: &str,
        synctoken: i64,
    ) -> Result<(Vec<AddressObject>, Vec<String>, i64), Error> {
        struct Row {
            object_id: String,
            synctoken: i64,
        }
        let changes = sqlx::query_as!(
            Row,
            r#"
                SELECT DISTINCT object_id, max(0, synctoken) as "synctoken!: i64" from addressobjectchangelog
                WHERE synctoken > ?
                ORDER BY synctoken ASC
            "#,
            synctoken
        )
        .fetch_all(&self.db)
        .await?;

        let mut objects = vec![];
        let mut deleted_objects = vec![];

        let new_synctoken = changes
            .last()
            .map(|&Row { synctoken, .. }| synctoken)
            .unwrap_or(0);

        for Row { object_id, .. } in changes {
            match self.get_object(principal, addressbook_id, &object_id).await {
                Ok(object) => objects.push(object),
                Err(Error::NotFound) => deleted_objects.push(object_id),
                Err(err) => return Err(err),
            }
        }

        Ok((objects, deleted_objects, new_synctoken))
    }

    #[instrument]
    async fn get_objects(
        &self,
        principal: &str,
        addressbook_id: &str,
    ) -> Result<Vec<AddressObject>, Error> {
        sqlx::query_as!(
            AddressObjectRow,
            "SELECT id, vcf FROM addressobjects WHERE principal = ? AND addressbook_id = ? AND deleted_at IS NULL",
            principal,
            addressbook_id
        )
        .fetch_all(&self.db)
        .await?
        .into_iter()
        .map(|row| row.try_into())
        .collect()
    }

    #[instrument]
    async fn get_object(
        &self,
        principal: &str,
        addressbook_id: &str,
        object_id: &str,
    ) -> Result<AddressObject, Error> {
        Ok(sqlx::query_as!(
            AddressObjectRow,
            "SELECT id, vcf FROM addressobjects WHERE (principal, addressbook_id, id) = (?, ?, ?)",
            principal,
            addressbook_id,
            object_id
        )
        .fetch_one(&self.db)
        .await?
        .try_into()?)
    }

    #[instrument]
    async fn put_object(
        &mut self,
        principal: String,
        addressbook_id: String,
        object: AddressObject,
    ) -> Result<(), Error> {
        let mut tx = self.db.begin().await?;

        let (object_id, vcf) = (object.get_id(), object.get_vcf());

        sqlx::query!(
            "REPLACE INTO addressobjects (principal, addressbook_id, id, vcf) VALUES (?, ?, ?, ?)",
            principal,
            addressbook_id,
            object_id,
            vcf
        )
        .execute(&mut *tx)
        .await?;

        log_object_operation(
            &mut tx,
            &principal,
            &addressbook_id,
            object_id,
            AddressbookChangeOperation::Add,
        )
        .await?;

        tx.commit().await?;
        Ok(())
    }

    #[instrument]
    async fn delete_object(
        &mut self,
        principal: &str,
        addressbook_id: &str,
        object_id: &str,
        use_trashbin: bool,
    ) -> Result<(), Error> {
        let mut tx = self.db.begin().await?;

        match use_trashbin {
            true => {
                sqlx::query!(
                    "UPDATE addressobjects SET deleted_at = datetime(), updated_at = datetime() WHERE (principal, addressbook_id, id) = (?, ?, ?)",
                    principal,
                    addressbook_id,
                    object_id
                )
                .execute(&mut *tx)
                .await?;
            }
            false => {
                sqlx::query!(
                    "DELETE FROM addressobjects WHERE addressbook_id = ? AND id = ?",
                    addressbook_id,
                    object_id
                )
                .execute(&mut *tx)
                .await?;
            }
        };
        log_object_operation(
            &mut tx,
            principal,
            addressbook_id,
            object_id,
            AddressbookChangeOperation::Delete,
        )
        .await?;
        tx.commit().await?;
        Ok(())
    }

    #[instrument]
    async fn restore_object(
        &mut self,
        principal: &str,
        addressbook_id: &str,
        object_id: &str,
    ) -> Result<(), Error> {
        let mut tx = self.db.begin().await?;

        sqlx::query!(
            r#"UPDATE addressobjects SET deleted_at = NULL, updated_at = datetime() WHERE (principal, addressbook_id, id) = (?, ?, ?)"#,
            principal,
            addressbook_id,
            object_id
        )
        .execute(&mut *tx)
        .await?;

        log_object_operation(
            &mut tx,
            principal,
            addressbook_id,
            object_id,
            AddressbookChangeOperation::Delete,
        )
        .await?;
        tx.commit().await?;
        Ok(())
    }
}
