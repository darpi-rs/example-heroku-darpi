use darpi::{middleware, Body, RequestParts};
use darpi_middleware::auth::UserRole;
use std::fmt;

#[middleware(Request)]
pub(crate) async fn roundtrip(
    #[request_parts] _rp: &RequestParts,
    #[body] _b: &Body,
    #[handler] msg: &str,
) -> Result<String, String> {
    Ok(format!("{} from roundtrip middleware", msg))
}

#[derive(Clone, PartialEq, PartialOrd)]
pub enum Role {
    User,
    Admin,
}

impl Role {
    pub fn from_str(role: &str) -> Role {
        match role {
            "Admin" => Role::Admin,
            _ => Role::User,
        }
    }
}

impl UserRole for Role {
    fn is_authorized(&self, other: &str) -> bool {
        let other = Self::from_str(other);
        self < &other
    }
}

impl fmt::Display for Role {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Role::User => write!(f, "User"),
            Role::Admin => write!(f, "Admin"),
        }
    }
}
