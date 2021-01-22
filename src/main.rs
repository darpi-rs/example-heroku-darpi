mod controller;
mod middleware;

use controller::{do_something, login};
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
    let address = format!("127.0.0.1:{}", 3000);
    app!({
        address: address,
        module: make_container => Container,
        // a set of global middleware that will be executed for every handler
        // the order matters and it's up to the user to apply them in desired order
        middleware: [body_size_limit(128), decompress()],
        bind: [
            {
                route: "/login",
                method: Method::POST,
                // the POST method allows this handler to have
                // Json<Name> as an argument
                handler: login
            },
            {
                route: "/hello_world/{name}",
                method: Method::GET,
                // the POST method allows this handler to have
                // Json<Name> as an argument
                handler: do_something
            }
        ],
    })
    .run()
    .await
}
