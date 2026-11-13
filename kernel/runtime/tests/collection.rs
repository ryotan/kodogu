#![forbid(unsafe_code)]

use kodogu_runtime::{Initialization, Initializer, initialize_all};
use kodogu_runtime::{collect, registered, submit};
use std::convert::Infallible;

struct Definition(u8);

collect!(Definition);
submit!(Definition(2));
submit!(Definition(3));

impl Initializer for Definition {
    type Context = String;
    type Instance = String;
    type Error = Infallible;
    type Future<'a> = Initialization<'a, String, Infallible>;

    fn initialize<'a>(&'a self, context: &'a String) -> Initialization<'a, String, Infallible> {
        Box::pin(async move { Ok(format!("{context}{}", self.0)) })
    }
}

#[test]
fn collects_definitions_without_requiring_collection_order() {
    let mut actual: Vec<_> = registered::<Definition>().map(|definition| definition.0).collect();
    actual.sort_unstable();
    assert_eq!(actual, vec![2, 3]);
}

#[tokio::test]
async fn static_definitions_accept_an_application_owned_context() {
    let context = String::from("local-");
    let mut instances = initialize_all(registered::<Definition>(), &context).await.unwrap();
    instances.sort();
    assert_eq!(instances, vec!["local-2", "local-3"]);
    assert_eq!(context, "local-");
}
