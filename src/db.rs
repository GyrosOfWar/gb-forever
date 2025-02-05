use std::time::Instant;

use color_eyre::eyre::Context;
use tracing::info;

use crate::{ia::MetadataItem, Result};

pub struct Database {
    pool: sqlx::PgPool,
}

impl Database {
    pub async fn connect(url: &str) -> Result<Self> {
        let pool = sqlx::PgPool::connect(url).await?;
        sqlx::migrate!("./migrations").run(&pool).await?;
        Ok(Self { pool })
    }

    pub async fn insert_items(&self, items: &[MetadataItem]) -> Result<()> {
        if items.is_empty() {
            return Ok(());
        }
        let start = Instant::now();
        let mut tx = self
            .pool
            .begin()
            .await
            .wrap_err("failed to start transaction")?;
        for item in items {
            sqlx::query!(
                r#"
                INSERT INTO gb_videos(
                    "date",
                    "description",
                    title,
                    item_size,
                    identifier,
                    external_identifier,
                    collections,
                    creator
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
                ON CONFLICT (identifier) DO NOTHING
                "#,
                item.date,
                item.description,
                item.title,
                item.item_size.map(|v| v as i64),
                item.identifier,
                item.external_identifier,
                item.collections.as_ref().map(|c| c.as_slice()),
                item.creator.as_deref()
            )
            .execute(&mut *tx)
            .await
            .wrap_err("failed to insert item")?;
        }
        tx.commit().await.wrap_err("failed to commit transaction")?;

        let elapsed = start.elapsed();
        info!("inserted {} items in {:?}", items.len(), elapsed);
        Ok(())
    }
}
