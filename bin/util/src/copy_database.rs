use anyhow::Context;
use futures_util::{TryStreamExt, future::join_all};
use mungos::{
  init::MongoBuilder,
  mongodb::bson::{Document, RawDocumentBuf},
};
use serde::Deserialize;

#[derive(Deserialize)]
struct Env {
  /// Provide the source mongo uri to copy from
  source_uri: String,
  /// Provide the source db name to copy from.
  /// Default: komodo
  #[serde(default = "default_db_name")]
  source_db_name: String,
  /// Provide the source mongo uri to copy to
  target_uri: String,
  /// Provide the target db name to copy to.
  /// Default: komodo
  #[serde(default = "default_db_name")]
  target_db_name: String,
}

fn default_db_name() -> String {
  String::from("komodo")
}

/// 10 MiB
const FLUSH_BYTES: usize = 10 * 1024 * 1024;

pub async fn main() -> anyhow::Result<()> {
  let env = envy::from_env::<Env>()?;

  let source_db = MongoBuilder::default()
    .uri(env.source_uri)
    .build()
    .await
    .context("Invalid SOURCE_URI")?
    .database(&env.source_db_name);
  let target_db = MongoBuilder::default()
    .uri(env.target_uri)
    .build()
    .await
    .context("Invalid SOURCE_URI")?
    .database(&env.target_db_name);

  let mut handles = Vec::new();

  for collection in source_db
    .list_collection_names()
    .await
    .context("Failed to list collections on source db")?
  {
    let source = source_db.collection::<RawDocumentBuf>(&collection);
    let target = target_db.collection::<RawDocumentBuf>(&collection);
    handles.push(tokio::spawn(async move {
      let res = async {
        let mut buffer = Vec::<RawDocumentBuf>::new();
        let mut size_bytes = 0;
        let mut cursor = source
          .find(Document::new())
          .await
          .context("Failed to query source collection")?;
        while let Some(doc) = cursor
          .try_next()
          .await
          .context("Failed to get next document")?
        {
          size_bytes += doc.as_bytes().len();
          buffer.push(doc);
          if size_bytes >= FLUSH_BYTES {
            target
              .insert_many(&buffer)
              .await
              .context("Failed to flush documents")?;
            size_bytes = 0;
            buffer.clear();
          }
        }
        if !buffer.is_empty() {
          target
            .insert_many(&buffer)
            .await
            .context("Failed to flush documents")?;
        }
        anyhow::Ok(())
      }
      .await;
      match res {
        Ok(_) => {
          info!("Finished copying {collection} collection");
        }
        Err(e) => {
          error!("Failed to copy {collection} collection | {e:#}")
        }
      }
    }));
  }

  join_all(handles).await;

  Ok(())
}
