mod controller;
mod middleware;

use controller::{do_something, home, important, login};
use darpi::{app, Method};
use darpi_middleware::auth::*;
use darpi_middleware::{body_size_limit, compression::decompress};
use shaku::module;

module! {
    pub Container {
        components = [JwtAlgorithmProviderImpl, JwtSecretProviderImpl, TokenExtractorImpl, JwtTokenCreatorImpl],
        providers = [],
    }
}

pub(crate) fn make_container() -> Container {
    let module = Container::builder()
        .with_component_parameters::<JwtSecretProviderImpl>(JwtSecretProviderImplParameters {
            secret: "my secret".to_string(),
        })
        .with_component_parameters::<JwtAlgorithmProviderImpl>(JwtAlgorithmProviderImplParameters {
            algorithm: Algorithm::ES256,
        })
        .build();
    module
}

#[tokio::main]
async fn main() -> Result<(), darpi::Error> {
    let port = std::env::var("PORT").unwrap();
    let address = "0.0.0.0:".to_owned() + &port;
    app!({
        address: address,
        container: {
            factory: make_container(),
            type: Container
        },
        // a set of global middleware that will be executed for every handler
        // the order matters and it's up to the user to apply them in desired order
        middleware: {
            request: [body_size_limit(128), decompress()]
        },
        handlers: [
            {
                route: "/",
                method: Method::GET,
                handler: home
            },
            {
                route: "/login",
                method: Method::POST,
                handler: login
            },
            {
                route: "/hello_world/{name}",
                method: Method::GET,
                handler: do_something
            },
            {
                route: "/important",
                method: Method::POST,
                handler: important
            }
        ]
    })
    .run()
    .await
}
