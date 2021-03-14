use crate::db::ServerDb;
use anyhow::Result;
use bollard::{
    container::{self, CreateContainerOptions},
    models::{ContainerConfig, HostConfig, RestartPolicy, RestartPolicyNameEnum},
    Docker,
};
use bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, env};
use tokio::{
    fs::File,
    io::{AsyncRead, AsyncReadExt},
};
use url::Url;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Server {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub name: String,
    pub version: String,
    pub mods: Vec<Url>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewServer {
    pub name: String,
    pub version: String,
    pub mods: Vec<Url>,
}

impl NewServer {
    pub async fn create(self, app_name: &str, db: &impl ServerDb) -> Result<ObjectId> {
        let id = db.insert(self.clone()).await?;
        let config = {
            let combined = CreateConfig::load(
                &mut File::open("fabric.toml").await?,
                app_name,
                &id.to_hex(),
            )
            .await?;
            let mut config: container::Config<_> = combined.main.into();
            config.host_config = combined.host.or(Some(Default::default())).map(|mut host| {
                host.restart_policy = host.restart_policy.or(Some(RestartPolicy {
                    name: Some(RestartPolicyNameEnum::UNLESS_STOPPED),
                    ..Default::default()
                }));
                host
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

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct CreateConfig {
    main: ContainerConfig,
    host: Option<HostConfig>,
}

impl CreateConfig {
    async fn load<R: AsyncRead + Unpin + ?Sized>(
        src: &mut R,
        app_name: &str,
        id: &str,
    ) -> Result<Self> {
        let mut vars = HashMap::<&str, &str>::new();
        let env_vars: Vec<_> = env::vars().collect();
        for (key, val) in &env_vars {
            vars.insert(key, val);
        }
        vars.insert("app", app_name);
        vars.insert("id", id);
        let read = {
            let mut buf = String::new();
            src.read_to_string(&mut buf).await?;
            buf
        };
        Ok(toml::from_str(&replace_vars(&read, &vars))?)
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

    #[tokio::test]
    async fn test_load_config() {
        let app_name = "servery";
        let id = ObjectId::new().to_hex();

        let data = r#"Main = {} 
Host = { Binds = ["{app}_{id}_data:/data"] }"#;
        assert!(CreateConfig::load(&mut data.as_bytes(), app_name, &id)
            .await
            .unwrap()
            .host
            .unwrap()
            .binds
            .unwrap()
            .contains(&format!("{}_{}_data:/data", app_name, id)));
    }

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
            "Hi {Name}, this is Peter31. {31{{}}{",
            replace_vars("Hi {Name}, this is {ME}{age}. {{age}{{}}{", &vars)
        );
    }
}
