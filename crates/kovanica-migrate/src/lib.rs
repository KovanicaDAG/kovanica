use anyhow::Result;

pub trait Migration {
    fn id(&self) -> &'static str;
    fn description(&self) -> &'static str;
    fn up(&self, _data_dir: &std::path::Path, dry_run: bool) -> Result<()>;
}

pub struct Registry {
    pub migrations: Vec<Box<dyn Migration>>,
}

impl Registry {
    pub fn default() -> Self {
        Self {
            migrations: vec![Box::new(NoopMigration)],
        }
    }
}

struct NoopMigration;
impl Migration for NoopMigration {
    fn id(&self) -> &'static str {
        "0001-noop"
    }
    fn description(&self) -> &'static str {
        "No-op placeholder migration"
    }
    fn up(&self, _data_dir: &std::path::Path, _dry_run: bool) -> Result<()> {
        Ok(())
    }
}
