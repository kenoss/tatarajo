use crate::backend::udev::SurfaceCompositionPolicy;
use std::path::PathBuf;

#[derive(Debug)]
pub(crate) struct EnvVar {
    /// Environment variables Without prefix.
    pub generic: EnvVarGeneric,
    /// Environment variables prefixed with `TATARAJO_`
    pub tatarajo: EnvVarTatarajo,
}

#[derive(Debug, serde::Deserialize)]
pub(crate) struct EnvVarGeneric {
    pub display: Option<String>,
    pub wayland_display: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
pub(crate) struct EnvVarTatarajo {
    /// Prevent auto detection and use designated DRM device node.
    ///
    /// Both primary node (e.g. /dev/dri/card0) and render node (e.g. /dev/dri/renderD128) are
    /// available. Tatarajo infers corresponding primary/render nodes.
    pub drm_device_node: Option<PathBuf>,
    #[serde(default = "default_bool::<false>")]
    pub disable_10bit: bool,
    #[serde(default = "Default::default")]
    pub surface_composition_policy: SurfaceCompositionPolicy,
    #[serde(default = "Default::default")]
    pub xkb_config: Option<String>,
}

// https://github.com/serde-rs/serde/issues/1030
// TODO(https://github.com/serde-rs/serde/issues/368): Use literal once default literals is supported.
const fn default_bool<const V: bool>() -> bool {
    V
}

impl EnvVar {
    pub fn load() -> eyre::Result<Self> {
        Ok(Self {
            generic: envy::from_env()?,
            tatarajo: envy::prefixed("TATARAJO_").from_env()?,
        })
    }

    pub fn xkb_config(&self) -> eyre::Result<Option<XkbConfig>> {
        self.tatarajo
            .xkb_config
            .as_deref()
            .map(serde_json::from_str)
            .transpose()
            .map_err(|e| e.into())
    }
}

#[derive(Debug, serde::Deserialize)]
pub(crate) struct XkbConfig {
    pub layout: String,
    pub repeat_delay: u16,
    pub repeat_rate: u16,
}
