use darpi::{middleware, Body, Request};
use darpi_middleware::auth::{Claims, UserRole};
use log::info;
use std::convert::Infallible;
use std::fmt;

#[middleware(Request)]
pub(crate) async fn roundtrip(
    #[request] _rp: &Request<Body>,
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
        info!("required: {} given: {}", self, other);
        &other >= self
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
