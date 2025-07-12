use std::{collections::HashMap, hash::Hash};

use tokio::sync::RwLock;

#[derive(Default)]
pub struct Cache<K: PartialEq + Eq + Hash, T: Clone + Default> {
  cache: RwLock<HashMap<K, T>>,
}

impl<
  K: PartialEq + Eq + Hash + std::fmt::Debug + Clone,
  T: Clone + Default,
> Cache<K, T>
{
  #[instrument(level = "debug", skip(self))]
  pub async fn get(&self, key: &K) -> Option<T> {
    self.cache.read().await.get(key).cloned()
  }

  #[instrument(level = "debug", skip(self))]
  pub async fn get_or_insert_default(&self, key: &K) -> T {
    let mut lock = self.cache.write().await;
    match lock.get(key).cloned() {
      Some(item) => item,
      None => {
        let item: T = Default::default();
        lock.insert(key.clone(), item.clone());
        item
      }
    }
  }

  #[instrument(level = "debug", skip(self))]
  pub async fn get_list(&self) -> Vec<T> {
    let cache = self.cache.read().await;
    cache.values().cloned().collect()
  }

  #[instrument(level = "debug", skip(self))]
  pub async fn insert<Key>(&self, key: Key, val: T)
  where
    T: std::fmt::Debug,
    Key: Into<K> + std::fmt::Debug,
  {
    self.cache.write().await.insert(key.into(), val);
  }

  // #[instrument(level = "debug", skip(self, handler))]
  // pub async fn update_entry<Key>(
  //   &self,
  //   key: Key,
  //   handler: impl Fn(&mut T),
  // ) where
  //   Key: Into<K> + std::fmt::Debug,
  // {
  //   let mut cache = self.cache.write().await;
  //   handler(cache.entry(key.into()).or_default());
  // }

  // #[instrument(level = "debug", skip(self))]
  // pub async fn clear(&self) {
  //   self.cache.write().await.clear();
  // }

  #[instrument(level = "debug", skip(self))]
  pub async fn remove(&self, key: &K) {
    self.cache.write().await.remove(key);
  }
}

// impl<
//   K: PartialEq + Eq + Hash + std::fmt::Debug + Clone,
//   T: Clone + Default + Busy,
// > Cache<K, T>
// {
//   #[instrument(level = "debug", skip(self))]
//   pub async fn busy(&self, id: &K) -> bool {
//     match self.get(id).await {
//       Some(state) => state.busy(),
//       None => false,
//     }
//   }
// }
