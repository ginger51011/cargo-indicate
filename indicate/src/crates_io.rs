//! Client and methods for retrieving information from the crates.io API
//!
//! _Note_: Due to the crates.io crawler policy, the amount of requests that
//! can be made is limited. [`CratesIoClient`] attempts to make this less
//! noticeable with caching and doing large fetches, but please keep this in
//! mind.
//! 
//! See https://crates.io/policies#crawlers for more information.

use std::{collections::HashMap, time::Duration};

use crates_io_api::{SyncClient, FullCrate, FullVersion};

use crate::NameVersion;

/// Wrapper around a [`crates_io_api::SyncClient`], with added caching
pub struct CratesIoClient {
    client: SyncClient,

    /// Cache between crate name and information about it
    cache: HashMap<String, FullCrate>,
}

impl CratesIoClient {
    pub fn new(user_agent: &str, rate_limit: Duration) -> Self {
        let client = SyncClient::new(user_agent, rate_limit).unwrap_or_else(|e| {
            panic!("could not create CratesIoClient due to error: {e}");
        });

        Self {
            client,
            cache: HashMap::new(),
        }
    }

    pub fn full_crate(crate_name: &str) -> &FullCrate {
        todo!()
    }

    pub fn full_version(name_version: &NameVersion) -> &FullVersion {
        todo!()
    }

    /// Retrieves the total amount of downloads for a crate, all versions
    ///
    /// # See also
    /// [`version_downloads`](CratesIoClient::version_downloads)
    pub fn total_downloads(crate_name: &str) -> u64 {
        todo!()
    }

    /// Retrieves the total amount of downloads for a specific crate version
    ///
    /// # See also
    /// [`total_downloads`](CratesIoClient::total_downloads)
    pub fn version_downloads(name_version: &NameVersion) -> u64 {
        todo!()
    }
}

impl Default for CratesIoClient {
    fn default() -> Self {
        let user_agent = std::env::var("USER_AGENT")
            .expect("USER_AGENT environment variable not set");
        Self::new(&user_agent, Duration::from_secs(1))
    }
}