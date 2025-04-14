use std::{
  collections::HashMap,
  sync::{Mutex, OnceLock},
};

use komodo_client::entities::{ResourceTarget, procedure::Procedure};
use tokio_util::sync::CancellationToken;

type ProcedureId = String;
type CancelMap = HashMap<ProcedureId, CancellationToken>;

fn cancels() -> &'static Mutex<CancelMap> {
  static CANCELS: OnceLock<Mutex<CancelMap>> = OnceLock::new();
  CANCELS.get_or_init(Default::default)
}

pub fn cancel_schedule(procedure_id: &str) {
  let cancel = cancels().lock().unwrap().remove(procedure_id);
  if let Some(cancel) = cancel {
    cancel.cancel();
  }
}

/// Re/spawns the schedule for the given procedure
pub async fn maybe_spawn_schedule(procedure: &Procedure) {
  // Cancel any existing schedule for the procedure
  cancel_schedule(&procedure.id);

  if procedure.config.schedule.is_empty() {
    return;
  }

  

  let cancel = CancellationToken::new();
  cancels()
    .lock()
    .unwrap()
    .insert(procedure.id.clone(), cancel.clone());
}
