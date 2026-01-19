use komodo_client::entities::stats::SystemProcess;
use mogh_resolver::Resolve;
use serde::{Deserialize, Serialize};

//

#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[response(Vec<SystemProcess>)]
#[error(anyhow::Error)]
pub struct GetSystemProcesses {}

//
