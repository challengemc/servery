use crate::db::ServerDb;
use anyhow::Result;
use bollard::{
    container::{self, CreateContainerOptions},
    models::{ContainerConfig, HostConfig, RestartPolicy, RestartPolicyNameEnum},
    Docker,
};
use bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::fs;
use url::Url;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Server {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub name: String,
    pub version: String,
    pub mods: Vec<Url>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct NewServer {
    pub name: String,
    pub version: String,
    pub mods: Vec<Url>,
}

impl NewServer {
    pub async fn create(self, app_name: &str, db: &impl ServerDb) -> Result<ObjectId> {
        let id = db.insert(self.clone()).await?;
        let config = {
            let mut vars = HashMap::new();
            vars.insert("app", app_name);
            vars.insert("name", &self.name);
            let config = replace_vars(&fs::read_to_string("fabric.toml").await?, &vars);
            let mut config: container::Config<_> =
                toml::from_str::<ContainerConfig>(&config)?.into();
            config.host_config = Some(HostConfig {
                restart_policy: Some(RestartPolicy {
                    name: Some(RestartPolicyNameEnum::UNLESS_STOPPED),
                    ..Default::default()
                }),
                ..Default::default()
            });
            config
        };
        let docker = Docker::connect_with_local_defaults()?;
        docker
            .create_container(
                Some(CreateContainerOptions { name: self.name }),
                config.into(),
            )
            .await?;
        Ok(id)
    }
}

fn replace_vars(string: &str, vars: &HashMap<&str, &str>) -> String {
    let mut new = String::with_capacity(string.len());
    let mut split = string.split('{');
    new.push_str(split.next().unwrap());
    for part in split {
        let mut push_part = || {
            new.push('{');
            new.push_str(part);
        };
        if let Some(end) = part.find('}') {
            if let Some(val) = vars.get(&part[..end]) {
                new.push_str(val);
                new.push_str(&part[end + 1..]);
            } else {
                push_part();
            }
        } else {
            push_part();
        }
    }
    new
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_replace_vars() {
        let vars = {
            let mut vars = HashMap::new();
            vars.insert("name", "Mike");
            vars.insert("ME", "Peter");
            vars.insert("age", "31");
            vars
        };
        assert_eq!(
            "Hi Mike, this is Peter. I am 31y/o",
            replace_vars("Hi {name}, this is {ME}. I am {age}y/o", &vars)
        );
        assert_eq!(
            "Hi {Name}, this is Peter. {31{{}}{",
            replace_vars("Hi {Name}, this is {ME}. {{age}{{}}{", &vars)
        );
    }
}
