#[derive(Debug)]
pub(crate) struct EnvVar {
    /// Environment variables Without prefix.
    pub generic: EnvVarGeneric,
    /// Environment variables prefixed with `SABINIWM_`
    pub sabiniwm: EnvVarSabiniwm,
}

#[derive(Debug, serde::Deserialize)]
pub(crate) struct EnvVarGeneric {
    pub display: Option<String>,
    pub wayland_display: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
pub(crate) struct EnvVarSabiniwm {
    pub drm_device: Option<String>,
    #[serde(default = "default_bool::<false>")]
    pub disable_10bit: bool,
    #[serde(default = "default_bool::<false>")]
    pub disable_drm_compositor: bool,
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
            sabiniwm: envy::prefixed("SABINIWM_").from_env()?,
        })
    }
}
