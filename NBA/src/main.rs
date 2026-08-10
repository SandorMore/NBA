#[allow(unused)]
#[macro_use] extern crate rocket;

#[rocket::main]
async fn main() -> ()
{
    rocket::build()
        .mount("/", routes![frontPage])
        .launch()
        .await;
}

#[get("/")]
fn frontPage() -> &'static str
{
    return "hello world";
}   