use std::path::{Path, PathBuf};

use anyhow::anyhow;
use parking_lot::Mutex;
use tokio::io::AsyncRead;

use crate::state::State;

pub struct Service {
    data_dir: PathBuf,
    state: Mutex<State>,
}

impl Service {
    pub fn new(data_dir: PathBuf, state: State) -> anyhow::Result<Self> {
        std::fs::create_dir_all(&data_dir)?;
        Ok(Self {
            data_dir,
            state: Mutex::new(state),
        })
    }
}

impl Service {
    pub async fn create(
        &self,
        mut body: impl AsyncRead + Unpin,
        auth: Option<(String, String)>,
    ) -> anyhow::Result<String> {
        if let Some((username, password)) = &auth {
            self.state
                .lock()
                .auth(username, password)
                .ok_or(anyhow!("Not authorized"))?;
        }
        let id = uuid::Uuid::new_v4().to_string();
        let path = self.data_dir.join(&id);
        let mut file = tokio::fs::File::create_new(path).await?;
        tokio::io::copy(&mut body, &mut file).await?;

        match &auth {
            None => {}
            Some((username, password)) => {
                self.state
                    .lock()
                    .auth_mut(username, password)
                    .ok_or(anyhow!("Not authorized"))?
                    .paste_ids
                    .push(id.clone());
            }
        };

        Ok(id)
    }

    pub async fn read(&self, id: &uuid::Uuid) -> anyhow::Result<tokio::fs::File> {
        let path = self.data_dir.join(id.to_string());
        let file = tokio::fs::File::open(path).await?;
        Ok(file)
    }

    pub async fn replace(
        &self,
        id: &uuid::Uuid,
        mut body: impl AsyncRead + Unpin,
        auth: Option<(String, String)>,
    ) -> anyhow::Result<()> {
        if let Some((username, password)) = &auth {
            let mut state = self.state.lock();
            let user = state
                .auth(username, password)
                .ok_or(anyhow!("Not authorized"))?;

            if !user.paste_ids.iter().any(|p| p == &id.to_string()) {
                anyhow::bail!("Paste not found");
            }
        }

        let path = self.data_dir.join(id.to_string());
        if !path.exists() {
            anyhow::bail!("Paste not found");
        }
        let mut file = tokio::fs::File::create(path).await?;
        tokio::io::copy(&mut body, &mut file).await?;

        Ok(())
    }

    pub fn delete(
        &self,
        id_to_delete: uuid::Uuid,
        username: &str,
        password: &str,
    ) -> anyhow::Result<()> {
        let id_to_delete = id_to_delete.to_string();
        let mut state = self.state.lock();
        let user = state
            .auth_mut(username, password)
            .ok_or(anyhow!("Not authorized"))?;
        let index = match user
            .paste_ids
            .iter()
            .enumerate()
            .find(|(_, id)| **id == id_to_delete)
        {
            None => anyhow::bail!("Paste not found"),
            Some((i, _)) => i,
        };
        std::fs::remove_file(self.data_dir.join(id_to_delete))?;
        user.paste_ids.remove(index);
        // TODO: clean up dangling entries if state serialization failed
        Ok(())
    }

    pub fn register_user(&self, username: &str, password: &str) -> anyhow::Result<()> {
        self.state.lock().create(username, password);
        Ok(())
    }

    pub fn list(&self, username: &str, password: &str) -> anyhow::Result<Vec<String>> {
        let state = self.state.lock();
        let user = state
            .auth(username, password)
            .ok_or(anyhow!("Not authorized"))?;
        Ok(user.paste_ids.iter().map(|s| s.clone()).collect())
    }

    pub fn dump_state(&self, path: &Path) -> anyhow::Result<()> {
        self.state.lock().dump(path)
    }
}
