use komodo_client::entities::stats::{
  SystemInformation, SystemProcess, SystemStats,
};
use resolver_api::Resolve;
use serde::{Deserialize, Serialize};

//

#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[response(SystemInformation)]
#[error(serror::Error)]
pub struct GetSystemInformation {}

//

#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[response(SystemStats)]
#[error(serror::Error)]
pub struct GetSystemStats {}

//

#[derive(Serialize, Deserialize, Debug, Clone, Resolve)]
#[response(Vec<SystemProcess>)]
#[error(serror::Error)]
pub struct GetSystemProcesses {}

//
