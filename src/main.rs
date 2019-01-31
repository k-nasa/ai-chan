#![feature(proc_macro_hygiene, decl_macro)]
#![feature(type_alias_enum_variants)]

mod hooks;

use rocket::{get, post, routes, Data};
use std::io::Read;

#[get("/")]
fn index() -> &'static str {
    // FIXME どうせなら使い方を出したほうが良いのでは？
    "Hello, world!"
}

fn main() {
    rocket().launch();
}

fn rocket() -> rocket::Rocket {
    rocket::ignite().mount("/", routes![index, github])
}

#[cfg(test)]
mod test {
    use super::*;
    use rocket::http::Status;
    use rocket::local::Client;

    #[test]
    fn test_index() {
        let client = Client::new(rocket()).expect("valid rocket instance");

        let mut response = client.get("/").dispatch();
        assert_eq!(response.status(), Status::Ok);
        assert_eq!(response.body_string(), Some("Hello, world!".into()));
    }
}
