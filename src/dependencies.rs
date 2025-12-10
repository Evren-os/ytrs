use crate::error::{Result, YtrsError};

pub fn check_dependencies(cmds: &[&str]) -> Result<()> {
    for cmd in cmds {
        if which::which(cmd).is_err() {
            return Err(YtrsError::MissingDependency((*cmd).to_string()));
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

    #[test]
    fn test_check_missing_command() {
        let result = check_dependencies(&["nonexistent_command_xyz"]);
        assert!(result.is_err());
    }
}
