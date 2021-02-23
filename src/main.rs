mod handlers;
mod jobs;
mod middleware;
mod models;
mod schema;
mod starwars;

#[macro_use]
extern crate diesel;

use async_graphql::{EmptyMutation, EmptySubscription, Schema};
use darpi::{app, Method};
use darpi_middleware::auth::*;
use darpi_middleware::{body_size_limit, compression::decompress};
use diesel::pg::PgConnection;
use diesel::r2d2::{self, ConnectionManager};
use dotenv::dotenv;
use handlers::{create_user, get_user, home, login};
use jobs::*;
use jsonwebtoken::{DecodingKey, EncodingKey};
use shaku::module;
use shaku::*;
use starwars::*;

pub trait DbPoolGetter: Interface {
    fn pool(&self) -> &DbPool;
}

type DbPool = r2d2::Pool<ConnectionManager<PgConnection>>;

#[derive(Component)]
#[shaku(interface = DbPoolGetter)]
pub struct DbPoolGetterImpl {
    #[shaku(default = unimplemented!())]
    db_pool: DbPool,
}

impl DbPoolGetter for DbPoolGetterImpl {
    fn pool(&self) -> &DbPool {
        &self.db_pool
    }
}

module! {
    pub Container {
        components = [
            JwtAlgorithmProviderImpl,
            JwtSecretProviderImpl,
            TokenExtractorImpl,
            JwtTokenCreatorImpl,
            SchemaGetterImpl,
            DbPoolGetterImpl
        ],
        providers = [],
    }
}

// our shaku container factory
// here we setup all our dependencies
// that can be referenced from handlers by the #[inject] attribute
pub(crate) fn make_container() -> Container {
    let schema = Schema::build(QueryRoot, EmptyMutation, EmptySubscription)
        .data(StarWars::new())
        .finish();

    // do not do this in a real application
    // or you will go to the pit of hell
    let secret = "my secret".as_ref();

    let connspec = std::env::var("DATABASE_URL").expect("DATABASE_URL");
    let manager = ConnectionManager::<PgConnection>::new(connspec);
    let db_pool = r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create pool.");

    let module = Container::builder()
        .with_component_parameters::<JwtSecretProviderImpl>(JwtSecretProviderImplParameters {
            encoding_key: EncodingKey::from_secret(secret),
            decoding_key: DecodingKey::from_secret(secret),
        })
        .with_component_parameters::<JwtAlgorithmProviderImpl>(JwtAlgorithmProviderImplParameters {
            algorithm: Algorithm::HS256,
        })
        .with_component_parameters::<SchemaGetterImpl>(SchemaGetterImplParameters { schema })
        .with_component_parameters::<DbPoolGetterImpl>(DbPoolGetterImplParameters { db_pool })
        .build();
    module
}

#[tokio::main]
async fn main() -> Result<(), darpi::Error> {
    dotenv().ok();
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
                route: "/user/{id}",
                method: Method::GET,
                handler: get_user
            },
            {
                route: "/user",
                method: Method::POST,
                handler: create_user
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
