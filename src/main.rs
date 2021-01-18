mod extractors;
mod middleware;

use darpi::{app, handler, Json, Method, Path, Query};
use extractors::*;
use middleware::access_control;
use middleware::UserRole::Admin;
use serde::{Deserialize, Serialize};
use shaku::module;

///////////// setup dependencies with shaku ///////////

module! {
    Container {
        components = [UserExtractorImpl],
        providers = [],
    }
}

fn make_container() -> Container {
    Container::builder().build()
}

////////////////////////

#[derive(Deserialize, Serialize, Debug, Path, Query)]
pub struct Name {
    name: String,
}

// Path<Name> is extracted from the registered path "/hello_world/{name}"
// and it is always mandatory. A request without "{name}" will result
// in the request path not matching the handler. It will either match another
// handler or result in an 404
// Option<Query<Name>> is extracted from the url query "?name=jason"
// it is optional, as the type suggests. To make it mandatory, simply
// remove the Option type. If there is a Query<T> in the handler and
// an incoming request url does not contain the query parameters, it will
// result in an error response
#[handler(Container)]
async fn hello_world(#[path] p: Name, #[query] q: Option<Name>) -> String {
    let other = q.map_or("nobody".to_owned(), |n| n.name);
    let response = format!("{} sends hello to {}", p.name, other);
    response
}

// the handler macro has 2 optional arguments
// the shaku container type and a collection of middlewares
// the enum variant `Admin` is coresponding to the middlewre `access_control`'s Expect<UserRole>
// Json<Name> is extracted from the request body
// failure to do so will result in an error response
#[handler(Container, [access_control(Admin)])]
async fn do_something(#[path] p: Name, #[body] payload: Json<Name>) -> String {
    let response = format!("{} sends hello to {}", p.name, payload.name);
    response
}

#[tokio::main]
async fn main() -> Result<(), darpi::Error> {
    let port = std::env::var("PORT").unwrap();
    let address = "0.0.0.0:".to_owned() + &port;
    app!({
        address: address,
        module: make_container => Container,
        middleware: [],
        bind: [
            {
                route: "/hello_world/{name}",
                //todo if user does not specify a method
                // let the user handle all methods on the same route
                // with a single handler
                method: Method::GET,
                handler: hello_world,
            },
            {
                route: "/hello_world/{name}",
                method: Method::POST,
                // the POST method allows this handler to have
                // Json<Name> as an argument
                handler: do_something
            },
        ],
    })
    .run()
    .await
}
