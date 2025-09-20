use std::path::PathBuf;

use clap::Parser;
use ichwilldich_lib::{FromReqExtension, config::BaseConfig};

use crate::s3::storage::StorageType;

#[derive(Parser, Clone, FromReqExtension)]
pub struct Config {
  #[command(flatten)]
  pub base: BaseConfig,

  // storage
  #[clap(short, long, env, default_value = "no-raid")]
  pub storage_type: StorageType,
  #[clap(short = 'p', long, env)]
  pub storage_path: PathBuf,

  // s3
  #[clap(long, env, default_value = "9000")]
  pub s3_port: u16,
}
