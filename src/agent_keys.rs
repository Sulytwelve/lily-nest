//! Authorized Ed25519 keys. Named keys require `kid`; old tokens use only `legacy`.
use std::{collections::BTreeMap, path::Path};

use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode, decode_header};
use serde::Deserialize;

use crate::model::AuthClaims;

pub struct AgentKeys(BTreeMap<String, AuthorizedKey>);

struct AuthorizedKey {
    name: String,
    enabled: bool,
    key: DecodingKey,
}

#[derive(Deserialize, Default)]
#[serde(deny_unknown_fields)]
struct KeyFile {
    #[serde(default)]
    keys: Vec<KeyConfig>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct KeyConfig {
    id: String,
    #[serde(default)]
    name: String,
    #[serde(default = "enabled_default")]
    enabled: bool,
    public_key: String,
}

fn enabled_default() -> bool {
    true
}

fn optional_file(path: &Path, required: bool) -> Result<Option<String>, String> {
    match std::fs::read_to_string(path) {
        Ok(contents) => Ok(Some(contents)),
        Err(error) if !required && error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(_) => Err(format!("cannot read Agent key file {}", path.display())),
    }
}

impl AgentKeys {
    pub fn load() -> Result<Self, String> {
        let configured = std::env::var_os("LILY_AGENT_KEYS_FILE");
        let path = configured
            .as_deref()
            .map(Path::new)
            .unwrap_or(Path::new(".agent-keys.toml"));
        let contents = optional_file(path, configured.is_some())?;
        let legacy = match std::env::var("LILY_AGENT_PUB_KEY") {
            Ok(pem) => Some(pem),
            Err(std::env::VarError::NotPresent) => optional_file(Path::new(".agent.pub"), false)?,
            Err(_) => return Err("LILY_AGENT_PUB_KEY must contain UTF-8 PEM".into()),
        };
        Self::parse(contents.as_deref(), legacy.as_deref())
    }

    fn parse(contents: Option<&str>, legacy: Option<&str>) -> Result<Self, String> {
        // Do not include TOML errors: they can quote configuration contents.
        let config: KeyFile = match contents {
            Some(contents) => {
                toml::from_str(contents).map_err(|_| "invalid Agent key configuration")?
            }
            None => KeyFile::default(),
        };
        let mut keys = BTreeMap::new();
        for config in config.keys {
            if config.id.is_empty()
                || config.id.len() > 64
                || !config
                    .id
                    .bytes()
                    .all(|c| c.is_ascii_alphanumeric() || matches!(c, b'-' | b'_'))
                || config.id == "legacy"
            {
                return Err("Agent key id must be 1-64 ASCII letters, digits, '-' or '_' and cannot be 'legacy'".into());
            }
            let key = DecodingKey::from_ed_pem(config.public_key.as_bytes())
                .map_err(|_| format!("invalid Ed25519 public key for {}", config.id))?;
            if keys
                .insert(
                    config.id,
                    AuthorizedKey {
                        name: config.name,
                        enabled: config.enabled,
                        key,
                    },
                )
                .is_some()
            {
                return Err("duplicate Agent key id".into());
            }
        }
        if let Some(pem) = legacy {
            let key = DecodingKey::from_ed_pem(pem.as_bytes())
                .map_err(|_| "invalid legacy Ed25519 public key")?;
            keys.insert(
                "legacy".into(),
                AuthorizedKey {
                    name: "legacy".into(),
                    enabled: true,
                    key,
                },
            );
        }
        Ok(Self(keys))
    }

    pub fn verify(&self, token: &str) -> Option<AuthClaims> {
        let header = decode_header(token).ok()?;
        if header.alg != Algorithm::EdDSA {
            return None;
        }
        let id = header.kid.as_deref().unwrap_or("legacy");
        let key = self.0.get(id).filter(|key| key.enabled)?;
        let claims = decode::<AuthClaims>(token, &key.key, &Validation::new(Algorithm::EdDSA))
            .ok()?
            .claims;
        tracing::debug!(key_id = id, key_name = %key.name, "Agent signature verified");
        Some(claims)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_invalid_configuration_without_exposing_pem() {
        assert_eq!(
            AgentKeys::parse(Some("private_secret = 'never-log-me'"), None)
                .err()
                .unwrap(),
            "invalid Agent key configuration"
        );
        for id in ["", "../key", "legacy", "with space"] {
            let config = format!("[[keys]]\nid = {id:?}\npublic_key = 'bad'");
            assert!(AgentKeys::parse(Some(&config), None).is_err());
        }
        assert!(AgentKeys::parse(None, Some("bad")).is_err());
        assert!(
            AgentKeys::parse(None, None)
                .unwrap()
                .verify("bad")
                .is_none()
        );
    }
}
