use std::{
  path::{Path, PathBuf},
  str::FromStr,
};

/// NOTE. This function will panic if file is non-None and fails to read file contents
pub fn maybe_read_item_from_file<T: FromStrDebugErr>(
  var_file: Option<PathBuf>,
  var: Option<T>,
) -> Option<T> {
  let Some(path) = var_file else { return var };
  let res = std::fs::read_to_string(&path)
    .map_err(|err| Error::<T>::ReadFileError {
      path: path.clone(),
      err,
    })
    .unwrap();
  let res = T::from_str(res.trim())
    .map_err(|err| Error::<T>::ParseValueError {
      path,
      err: err.into(),
    })
    .unwrap();
  Some(res)
}

/// NOTE. This function will panic if file is non-None and fails to read file contents
pub fn maybe_read_list_from_file<T: FromStrDebugErr>(
  var_file: Option<PathBuf>,
  var: Option<Vec<T>>,
) -> Option<Vec<T>> {
  let Some(path) = var_file else { return var };
  Some(parse_list_from_file(&path).unwrap())
}

pub trait FromStrDebugErr: FromStr + std::fmt::Debug {
  type Error: std::fmt::Debug + From<Self::Err>;
}

impl FromStrDebugErr for String {
  type Error = <String as FromStr>::Err;
}

impl FromStrDebugErr for i64 {
  type Error = <i64 as FromStr>::Err;
}

#[derive(Debug, thiserror::Error)]
enum Error<T: std::fmt::Debug + FromStrDebugErr> {
  #[error("Failed to read file contents from {path:?} | {err:?}")]
  ReadFileError { path: PathBuf, err: std::io::Error },
  #[error("Failed to parse file contents from {path:?} | {err:?}")]
  ParseValueError { path: PathBuf, err: T::Error },
}

fn parse_list_from_file<T: FromStrDebugErr>(
  path: &Path,
) -> Result<Vec<T>, Error<T>> {
  std::fs::read_to_string(path)
    .map_err(|err| Error::ReadFileError {
      path: path.to_path_buf(),
      err,
    })?
    .split(',')
    .map(str::trim)
    .map(|s| {
      T::from_str(s).map_err(|err| Error::ParseValueError {
        path: path.to_path_buf(),
        err: err.into(),
      })
    })
    .collect::<Result<Vec<_>, Error<_>>>()
}
