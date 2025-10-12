use figment::{
  Figment,
  providers::{Env, Serialized},
};
use serde::{Deserialize, Serialize};
use tracing::instrument;
use url::Url;

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct SavedConfig {
  #[serde(default, flatten)]
  pub oidc: SSOConfig,
}

impl SavedConfig {
  #[instrument]
  pub fn parse() -> Self {
    let config = Figment::new()
      .merge(Serialized::defaults(Self::default()))
      .merge(Env::raw().global());

    config.extract().expect("failed to load configuration")
  }
}

#[derive(Clone, Debug)]
pub struct MergedConfig {
  pub oidc: MergedSSOConfig,
}

#[derive(Serialize, Deserialize, Default, Clone, Debug)]
pub struct SSOConfig {
  sso_instant_redirect: Option<bool>,

  oidc_client_id: Option<String>,
  oidc_client_secret: Option<String>,
  oidc_url: Option<Url>,
  oidc_scope: Option<String>,
}

#[derive(Clone, Debug)]
pub struct MergedSSOConfig {
  pub sso_instant_redirect: ConfigValue<bool>,
  pub oidc_client_id: ConfigValue<String>,
  pub oidc_client_secret: ConfigValue<String>,
  pub oidc_url: ConfigValue<Url>,
  pub oidc_scope: ConfigValue<String>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
#[serde(tag = "type", content = "value")]
pub enum ConfigValue<T> {
  Env(T),
  UI(T),
  None,
}

impl<T> ConfigValue<T> {
  pub fn value(&self) -> Option<&T> {
    match self {
      ConfigValue::Env(v) => Some(v),
      ConfigValue::UI(v) => Some(v),
      ConfigValue::None => None,
    }
  }

  fn from_value(env: Option<T>, ui: Option<T>) -> Self {
    if let Some(v) = env {
      ConfigValue::Env(v)
    } else if let Some(v) = ui {
      ConfigValue::UI(v)
    } else {
      ConfigValue::None
    }
  }
}

impl SavedConfig {
  pub(crate) fn merge(self, ui: Self) -> MergedConfig {
    MergedConfig {
      oidc: self.oidc.merge(ui.oidc),
    }
  }
}

impl MergedConfig {
  pub(super) fn to_ui(&self) -> SavedConfig {
    SavedConfig {
      oidc: self.oidc.to_ui(),
    }
  }
}

impl SSOConfig {
  pub(crate) fn merge(self, ui: Self) -> MergedSSOConfig {
    MergedSSOConfig {
      sso_instant_redirect: ConfigValue::from_value(
        self.sso_instant_redirect,
        ui.sso_instant_redirect,
      ),
      oidc_client_id: ConfigValue::from_value(self.oidc_client_id, ui.oidc_client_id),
      oidc_client_secret: ConfigValue::from_value(self.oidc_client_secret, ui.oidc_client_secret),
      oidc_url: ConfigValue::from_value(self.oidc_url, ui.oidc_url),
      oidc_scope: ConfigValue::from_value(self.oidc_scope, ui.oidc_scope),
    }
  }
}

impl MergedSSOConfig {
  pub(super) fn to_ui(&self) -> SSOConfig {
    SSOConfig {
      sso_instant_redirect: self.sso_instant_redirect.value().cloned(),
      oidc_client_id: self.oidc_client_id.value().cloned(),
      oidc_client_secret: self.oidc_client_secret.value().cloned(),
      oidc_url: self.oidc_url.value().cloned(),
      oidc_scope: self.oidc_scope.value().cloned(),
    }
  }
}

#[cfg(test)]
mod test {
  use super::*;

  #[test]
  fn test_merge() {
    let env = SavedConfig {
      oidc: SSOConfig {
        sso_instant_redirect: Some(true),
        oidc_client_id: Some("env_client_id".to_string()),
        oidc_client_secret: Some("env_client_secret".to_string()),
        oidc_url: None,
        oidc_scope: None,
      },
    };

    let ui = SavedConfig {
      oidc: SSOConfig {
        sso_instant_redirect: None,
        oidc_client_id: Some("ui_client_id".to_string()),
        oidc_client_secret: None,
        oidc_url: Some(Url::parse("https://ui.example.com").unwrap()),
        oidc_scope: None,
      },
    };

    let merged = env.merge(ui);

    assert_eq!(merged.oidc.sso_instant_redirect, ConfigValue::Env(true));
    assert_eq!(
      merged.oidc.oidc_client_id,
      ConfigValue::Env("env_client_id".to_string())
    );
    assert_eq!(
      merged.oidc.oidc_client_secret,
      ConfigValue::Env("env_client_secret".to_string())
    );
    assert_eq!(
      merged.oidc.oidc_url,
      ConfigValue::UI(Url::parse("https://ui.example.com").unwrap())
    );
    assert_eq!(merged.oidc.oidc_scope, ConfigValue::None);
  }

  #[test]
  fn test_to_ui() {
    let merged = MergedConfig {
      oidc: MergedSSOConfig {
        sso_instant_redirect: ConfigValue::Env(true),
        oidc_client_id: ConfigValue::UI("ui_client_id".to_string()),
        oidc_client_secret: ConfigValue::None,
        oidc_url: ConfigValue::Env(Url::parse("https://env.example.com").unwrap()),
        oidc_scope: ConfigValue::None,
      },
    };

    let ui = merged.to_ui();
    assert_eq!(ui.oidc.sso_instant_redirect, Some(true));
    assert_eq!(ui.oidc.oidc_client_id, Some("ui_client_id".to_string()));
    assert_eq!(ui.oidc.oidc_client_secret, None);
    assert_eq!(
      ui.oidc.oidc_url,
      Some(Url::parse("https://env.example.com").unwrap())
    );
    assert_eq!(ui.oidc.oidc_scope, None);
  }

  #[test]
  fn test_sso_instant_redirect() {
    unsafe {
      std::env::set_var("SSO_INSTANT_REDIRECT", "false");
    }
    let cfg = SavedConfig::parse();
    assert_eq!(cfg.oidc.sso_instant_redirect, Some(false));
  }

  #[test]
  fn test_oidc_client_id() {
    unsafe {
      std::env::set_var("OIDC_CLIENT_ID", "test_client_id");
    }
    let cfg = SavedConfig::parse();
    assert_eq!(cfg.oidc.oidc_client_id, Some("test_client_id".to_string()));
  }

  #[test]
  fn test_oidc_client_secret() {
    unsafe {
      std::env::set_var("OIDC_CLIENT_SECRET", "test_client_secret");
    }
    let cfg = SavedConfig::parse();
    assert_eq!(
      cfg.oidc.oidc_client_secret,
      Some("test_client_secret".to_string())
    );
  }

  #[test]
  fn test_oidc_url() {
    unsafe {
      std::env::set_var("OIDC_URL", "https://example.com");
    }
    let cfg = SavedConfig::parse();
    assert_eq!(
      cfg.oidc.oidc_url,
      Some(Url::parse("https://example.com").unwrap())
    );
  }

  #[test]
  fn test_oidc_scope() {
    unsafe {
      std::env::set_var("OIDC_SCOPE", "openid profile email");
    }
    let cfg = SavedConfig::parse();
    assert_eq!(
      cfg.oidc.oidc_scope,
      Some("openid profile email".to_string())
    );
  }

  #[test]
  fn test_config_value() {
    let env_value: ConfigValue<String> = ConfigValue::Env("env".to_string());
    let ui_value: ConfigValue<String> = ConfigValue::UI("ui".to_string());
    let none_value: ConfigValue<String> = ConfigValue::None;

    assert_eq!(env_value.value(), Some(&"env".to_string()));
    assert_eq!(ui_value.value(), Some(&"ui".to_string()));
    assert_eq!(none_value.value(), None);
  }

  #[test]
  fn test_config_value_from_value() {
    let env = ConfigValue::from_value(Some("env".to_string()), Some("ui".to_string()));
    let ui = ConfigValue::from_value(None, Some("ui".to_string()));
    let none: ConfigValue<()> = ConfigValue::from_value(None, None);

    assert_eq!(env, ConfigValue::Env("env".to_string()));
    assert_eq!(ui, ConfigValue::UI("ui".to_string()));
    assert_eq!(none, ConfigValue::None);
  }
}
