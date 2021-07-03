#![cfg(feature = "swarm")]
//! Install, create and manage plugins
pub mod data;
pub mod opts;

pub use data::*;
pub use opts::*;

use crate::{
    conn::Payload,
    util::url::{construct_ep, encoded_pair},
    Result,
};

use std::path::Path;

impl_api_ty!(Plugin => name: N);

impl<'docker> Plugin<'docker> {
    /// Inspects a named plugin's details.
    ///
    /// [Api Reference](https://docs.docker.com/engine/api/v1.41/#operation/PluginInspect)
    pub async fn inspect(&self) -> Result<PluginInfo> {
        self.docker
            .get_json(&format!("/plugins/{}/json", self.name))
            .await
    }

    async fn _remove(&self, force: bool) -> Result<PluginInfo> {
        let query = if force {
            Some(encoded_pair("force", true))
        } else {
            None
        };
        self.docker
            .delete_json(&construct_ep(format!("/plugins/{}", self.name), query))
            .await
    }

    /// Removes a plugin.
    ///
    /// [Api Reference](https://docs.docker.com/engine/api/v1.41/#operation/PluginDelete)
    pub async fn remove(&self) -> Result<PluginInfo> {
        self._remove(false).await
    }

    /// Forcefully remove a plugin. This may result in issues if the plugin is in use by a container.
    ///
    /// [Api Reference](https://docs.docker.com/engine/api/v1.41/#operation/PluginDelete)
    pub async fn force_remove(&self) -> Result<PluginInfo> {
        self._remove(true).await
    }

    /// Enable a plugin.
    ///
    /// [Api Reference](https://docs.docker.com/engine/api/v1.41/#operation/PluginEnable)
    pub async fn enable(&self, timeout: Option<u64>) -> Result<()> {
        let query = if let Some(timeout) = timeout {
            Some(encoded_pair("timeout", timeout))
        } else {
            None
        };
        self.docker
            .post(
                &construct_ep(format!("/plugins/{}/enable", self.name), query),
                Payload::empty(),
            )
            .await
            .map(|_| ())
    }

    /// Disable a plugin.
    ///
    /// [Api Reference](https://docs.docker.com/engine/api/v1.41/#operation/PluginDisable)
    pub async fn disable(&self) -> Result<()> {
        self.docker
            .post(&format!("/plugins/{}/disable", self.name), Payload::empty())
            .await
            .map(|_| ())
    }

    /// Push a plugin to the registry.
    ///
    /// [Api Reference](https://docs.docker.com/engine/api/v1.41/#operation/PluginPush)
    pub async fn push(&self) -> Result<()> {
        self.docker
            .post(&format!("/plugins/{}/push", self.name), Payload::empty())
            .await
            .map(|_| ())
    }

    /// Create a plugin from a tar archive on the file system. The `path` parameter is a path
    /// to the tar containing plugin rootfs and manifest.
    ///
    /// [Api Reference](https://docs.docker.com/engine/api/v1.41/#operation/PluginCreate)
    pub async fn create<P>(&self, path: P) -> Result<()>
    where
        P: AsRef<Path>,
    {
        self.docker
            .post(
                &format!("/plugins/{}/create", self.name),
                Payload::Text(path.as_ref().to_string_lossy().to_string()),
            )
            .await
            .map(|_| ())
    }
}

impl<'docker> Plugins<'docker> {
    /// Returns information about installed plugins.
    ///
    /// [Api Reference](https://docs.docker.com/engine/api/v1.41/#operation/PluginList)
    pub async fn list(&self, opts: &PluginListOpts) -> Result<Vec<PluginInfo>> {
        self.docker
            .get_json::<Vec<PluginInfo>>(&construct_ep("/plugins", opts.serialize()))
            .await
    }
}
