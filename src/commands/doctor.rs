use std::path::Path;

use anyhow::{Result, bail};

use super::Output;
use crate::diagnostics::{self, Check};
use crate::service::ServiceHost;

pub struct DoctorReport {
    checks: Vec<Check>,
}

impl DoctorReport {
    pub fn output(&self) -> Output {
        Output::lines(self.checks.iter().map(Check::line).collect())
    }

    pub fn verdict(&self) -> Result<()> {
        let failures = self.checks.iter().filter(|check| check.is_failure()).count();
        if failures > 0 {
            bail!("doctor found {failures} failing check(s)");
        }
        Ok(())
    }
}

pub async fn run(home: &Path) -> DoctorReport {
    let service = ServiceHost::of_this_machine().ok();
    DoctorReport {
        checks: diagnostics::diagnose(home, service.as_ref()).await,
    }
}
