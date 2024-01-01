use serde::{Deserialize, Serialize};

pub fn is_valid_user_role(role: &str) -> bool {
    match role {
        "team_manager" => true,
        "insurance" => true,
        _ => false,
    }
}

pub fn is_valid_username(username: &str) -> bool {
    username.len() > 3
}

pub(crate) fn is_valid_password(password: &str) -> bool {
    password.len() > 3
}


// Define the struct for the claims
#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct Claims {
    pub(crate) username: String,
    pub(crate) role: String,
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_valid_user_role() {
        assert!(is_valid_user_role("team_manager"));
        assert!(is_valid_user_role("insurance"));
        assert!(!is_valid_user_role("invalid_role"));
    }

    #[test]
    fn test_is_valid_username() {
        assert!(is_valid_username("validuser"));
        assert!(!is_valid_username("ali"));
    }

    #[test]
    fn test_is_valid_password() {
        assert!(is_valid_password("validpassword"));
        assert!(!is_valid_password("123"));
    }
}