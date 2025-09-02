use anyhow::Context;
use futures_util::{
  StreamExt, TryStreamExt, stream::FuturesUnordered,
};
use mungos::{
  bulk_update::{BulkUpdate, bulk_update_retry_too_big},
  mongodb::{
    Database,
    bson::{Document, doc},
  },
};
use tracing::{error, info};

pub async fn copy(
  source_db: &Database,
  target_db: &Database,
) -> anyhow::Result<()> {
  let mut handles = source_db
    .list_collection_names()
    .await
    .context("Failed to list collections on source db")?.into_iter().map(|collection| {
      let source = source_db.collection::<Document>(&collection);
      let target_db = target_db.clone();
      tokio::spawn(async move {
        let res = async {
          let mut buffer = Vec::<BulkUpdate>::new();
          // The update collection is bigger than others,
          // can hit the max bson limit on the bulk upsert call without this.
          let max_buffer = if collection == "Update" {
            1_000
          } else {
            10_000
          };
          let mut count = 0;
          let mut cursor = source
            .find(Document::new())
            .await
            .context("Failed to query source collection")?;
          while let Some(document) = cursor
            .try_next()
            .await
            .context("Failed to get next document")?
          {
            let Some(id) = document.get("_id").and_then(|id| id.as_object_id()) else {
              continue;
            };
            count += 1;
            buffer.push(BulkUpdate { query: doc! { "_id": id }, update: doc! { "$set": document } });
            if buffer.len() >= max_buffer {
              if let Err(e) = bulk_update_retry_too_big(&target_db, &collection, &buffer, true).await.context("Failed to flush documents")
              {
                error!("Failed to flush document batch in {collection} collection | {e:#}");
              };
              buffer.clear();
            }
          }
          if !buffer.is_empty() {
            bulk_update_retry_too_big(&target_db, &collection, &buffer, true)
              .await
              .context("Failed to flush documents")?;
          }
          anyhow::Ok(count)
        }
        .await;
        match res {
          Ok(count) => {
            if count > 0 {
              info!("Finished copying {collection} collection | Copied {count}");
            }
          }
          Err(e) => {
            error!("Failed to copy {collection} collection | {e:#}")
          }
        }
      })
    }).collect::<FuturesUnordered<_>>();

  loop {
    match handles.next().await {
      Some(Ok(())) => {}
      Some(Err(e)) => {
        error!("{e:#}");
      }
      None => break,
    }
  }

  info!("Finished copying database ✅");

  Ok(())
}
