use darpi::{middleware, Body, RequestParts};
use darpi_middleware::auth::{Claims, UserRole};
use std::convert::Infallible;
use std::fmt;

#[middleware(Request)]
pub(crate) async fn roundtrip(
    #[request_parts] _rp: &RequestParts,
    #[body] _b: &Body,
    #[handler] msg: impl AsRef<str> + Send + Sync + 'static,
) -> Result<String, Infallible> {
    let res = format!("{} from roundtrip middleware", msg.as_ref());
    Ok(res)
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
    fn is_authorized(&self, claims: &Claims) -> bool {
        let other = Self::from_str(claims.role());
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
