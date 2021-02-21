mod handlers;
mod jobs;
mod middleware;
mod starwars;

use async_graphql::{EmptyMutation, EmptySubscription, Schema};
use darpi::{app, Method};
use darpi_middleware::auth::*;
use darpi_middleware::{body_size_limit, compression::decompress};
use handlers::{do_something, home, important, login};
use jobs::*;
use jsonwebtoken::{DecodingKey, EncodingKey};
use shaku::module;
use starwars::*;

module! {
    pub Container {
        components = [
            JwtAlgorithmProviderImpl,
            JwtSecretProviderImpl,
            TokenExtractorImpl,
            JwtTokenCreatorImpl,
            SchemaGetterImpl
        ],
        providers = [],
    }
}

pub(crate) fn make_container() -> Container {
    let schema = Schema::build(QueryRoot, EmptyMutation, EmptySubscription)
        .data(StarWars::new())
        .finish();

    let secret = "my secret".as_ref();

    let module = Container::builder()
        .with_component_parameters::<JwtSecretProviderImpl>(JwtSecretProviderImplParameters {
            encoding_key: EncodingKey::from_secret(secret),
            decoding_key: DecodingKey::from_secret(secret),
        })
        .with_component_parameters::<JwtAlgorithmProviderImpl>(JwtAlgorithmProviderImplParameters {
            algorithm: Algorithm::HS256,
        })
        .with_component_parameters::<SchemaGetterImpl>(SchemaGetterImplParameters { schema })
        .build();
    module
}

#[tokio::main]
async fn main() -> Result<(), darpi::Error> {
    std::env::set_var("RUST_LOG", "info");
    env_logger::init();

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
        jobs: {
            response: [first_sync_job, first_sync_job1, first_sync_io_job]
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
            },
            //graphql
            {
                route: "/starwars",
                method: Method::POST,
                handler: starwars_post
            },
            {
                route: "/starwars",
                method: Method::GET,
                handler: starwars_get
            }
        ]
    })
    .run()
    .await
}
