use anyhow::{anyhow, Result};

pub fn check_dependencies(cmds: &[&str]) -> Result<()> {
    for cmd in cmds {
        if which::which(cmd).is_err() {
            return Err(anyhow!("{} is not installed or not found in PATH", cmd));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_existing_command() {
        let result = check_dependencies(&["sh"]);
        assert!(result.is_ok());
    }
}
