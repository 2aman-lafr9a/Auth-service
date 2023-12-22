use serde::Serialize;

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


// pub fn sanitize_input(input: String) -> String {
//     let re = Regex::new(r"[^a-zA-Z0-9]").unwrap();
//     let sanitized_input = re.replace_all(input.to_owned().as_str(), "").to_string();
//     sanitized_input
// }

// Define the struct for the claims
#[derive(Debug, Serialize)]
pub(crate) struct Claims {
    pub(crate) username: String,
    pub(crate) role: String,
}