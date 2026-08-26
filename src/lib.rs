pub mod constants {
    pub const CANDIDATES_DIR: &str = "candidates";
    pub const CANDIDATES_FILE: &str = "candidates";
    pub const CURRENT_DIR: &str = "current";
    pub const CURRENT_VERSION_FILE: &str = ".sdkman-current-version";
    pub const DEFAULT_SDKMAN_HOME: &str = ".sdkman";
    pub const SDKMAN_DIR_ENV_VAR: &str = "SDKMAN_DIR";
    pub const TMP_DIR: &str = "tmp";
    pub const VAR_DIR: &str = "var";
}

pub mod helpers {
    use colored::Colorize;
    use directories::UserDirs;
    use std::path::{Component, Path, PathBuf};
    use std::{env, fs, process};

    use crate::constants::{
        CANDIDATES_DIR, CANDIDATES_FILE, DEFAULT_SDKMAN_HOME, SDKMAN_DIR_ENV_VAR, VAR_DIR,
    };

    pub fn infer_sdkman_dir() -> PathBuf {
        env::var_os(SDKMAN_DIR_ENV_VAR)
            .filter(|path| !path.is_empty())
            .map(PathBuf::from)
            .unwrap_or_else(fallback_sdkman_dir)
    }

    fn fallback_sdkman_dir() -> PathBuf {
        UserDirs::new()
            .map(|dir| dir.home_dir().join(DEFAULT_SDKMAN_HOME))
            .unwrap()
    }

    pub fn read_file_content(path: &Path) -> Option<String> {
        fs::read_to_string(path)
            .ok()
            .map(|content| content.trim().to_owned())
            .filter(|content| !content.is_empty())
    }

    fn is_normal_path_segment(value: &str) -> bool {
        let mut components = Path::new(value).components();
        matches!(components.next(), Some(Component::Normal(_))) && components.next().is_none()
    }

    pub fn known_candidates(sdkman_dir: &Path) -> Vec<String> {
        let absolute_path = sdkman_dir.join(VAR_DIR).join(CANDIDATES_FILE);
        let content = match fs::read_to_string(&absolute_path) {
            Ok(content) => content,
            Err(_) => {
                eprintln!(
                    "cannot read SDKMAN candidates file {}",
                    absolute_path.display()
                );
                process::exit(1);
            }
        };

        let candidates: Vec<_> = content
            .split(',')
            .map(str::trim)
            .filter(|candidate| !candidate.is_empty())
            .map(str::to_owned)
            .collect();

        if candidates
            .iter()
            .any(|candidate| !is_normal_path_segment(candidate))
        {
            eprintln!(
                "SDKMAN candidates file {} contains an invalid candidate name",
                absolute_path.display()
            );
            process::exit(1);
        }

        candidates
    }

    pub fn validate_candidate(all_candidates: &[String], candidate: &str) {
        if !is_normal_path_segment(candidate)
            || !all_candidates.iter().any(|known| known == candidate)
        {
            eprintln!("{} is not a valid candidate.", candidate.bold());
            process::exit(1);
        }
    }

    pub fn validate_version_path(base_dir: &Path, candidate: &str, version: &str) -> PathBuf {
        if !is_normal_path_segment(candidate) || !is_normal_path_segment(version) {
            eprintln!(
                "{} {} is not installed on your system",
                candidate.bold(),
                version.bold()
            );
            process::exit(1);
        }

        let candidates_path = base_dir.join(CANDIDATES_DIR);
        let candidate_path = candidates_path.join(candidate);
        let version_path = candidate_path.join(version);
        let contained = (|| {
            let base_dir = fs::canonicalize(base_dir).ok()?;
            let candidates_path = fs::canonicalize(candidates_path).ok()?;
            let candidate_path = fs::canonicalize(candidate_path).ok()?;
            let version_path = fs::canonicalize(&version_path).ok()?;

            Some(
                candidates_path == base_dir.join(CANDIDATES_DIR)
                    && candidate_path == candidates_path.join(candidate)
                    && version_path == candidate_path.join(version),
            )
        })()
        .unwrap_or(false);

        if contained && version_path.is_dir() {
            version_path
        } else {
            eprintln!(
                "{} {} is not installed on your system",
                candidate.bold(),
                version.bold()
            );
            process::exit(1)
        }
    }
}

#[cfg(test)]
mod tests {
    use std::env;
    use std::ffi::OsString;
    use std::fs;
    #[cfg(unix)]
    use std::os::unix::ffi::OsStringExt;
    use std::path::PathBuf;

    use serial_test::serial;
    use tempfile::TempDir;

    use crate::constants::SDKMAN_DIR_ENV_VAR;
    use crate::helpers::infer_sdkman_dir;
    use crate::helpers::read_file_content;

    struct EnvVarGuard(Option<OsString>);

    impl EnvVarGuard {
        fn capture(name: &str) -> Self {
            Self(env::var_os(name))
        }
    }

    impl Drop for EnvVarGuard {
        fn drop(&mut self) {
            if let Some(value) = self.0.take() {
                env::set_var(SDKMAN_DIR_ENV_VAR, value);
            } else {
                env::remove_var(SDKMAN_DIR_ENV_VAR);
            }
        }
    }

    #[test]
    #[serial]
    fn should_infer_sdkman_dir_from_env_var() {
        let _guard = EnvVarGuard::capture(SDKMAN_DIR_ENV_VAR);
        let sdkman_dir = PathBuf::from("/home/someone/.sdkman");
        env::set_var(SDKMAN_DIR_ENV_VAR, &sdkman_dir);
        assert_eq!(sdkman_dir, infer_sdkman_dir());
    }

    #[test]
    #[serial]
    fn should_infer_fallback_dir_for_missing_or_empty_env_var() {
        let _guard = EnvVarGuard::capture(SDKMAN_DIR_ENV_VAR);
        let actual_sdkman_dir = directories::UserDirs::new()
            .unwrap()
            .home_dir()
            .join(".sdkman");
        for value in [None, Some("")] {
            match value {
                Some(value) => env::set_var(SDKMAN_DIR_ENV_VAR, value),
                None => env::remove_var(SDKMAN_DIR_ENV_VAR),
            }
            assert_eq!(actual_sdkman_dir, infer_sdkman_dir());
        }
    }

    #[cfg(unix)]
    #[test]
    #[serial]
    fn should_preserve_non_utf8_sdkman_dir() {
        let _guard = EnvVarGuard::capture(SDKMAN_DIR_ENV_VAR);
        let value = OsString::from_vec(vec![b'/', b't', b'm', b'p', b'/', 0x80]);
        let expected = PathBuf::from(value.clone());
        env::set_var(SDKMAN_DIR_ENV_VAR, value);
        assert_eq!(expected, infer_sdkman_dir());
    }

    #[test]
    fn should_read_trimmed_non_blank_content_only() {
        let temp_dir = TempDir::new().unwrap();
        for (name, content, expected) in [
            ("trimmed", Some(" 5.0.0\n"), Some("5.0.0")),
            ("blank", Some("  \n\t"), None),
            ("missing", None, None),
        ] {
            let path = temp_dir.path().join(name);
            if let Some(content) = content {
                fs::write(&path, content).unwrap();
            }
            assert_eq!(
                read_file_content(&path),
                expected.map(str::to_owned),
                "{name}"
            );
        }
    }
}
