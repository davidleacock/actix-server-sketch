use actix_web::{App, get, HttpResponse, HttpServer, Responder as ActixResponder};
use rand::Rng;
use libmdns::Responder;
use std::env;
use std::time::Duration;


#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let sensor_port = env::var("PORT").unwrap_or(String::from("8080"));
    let sensor_name = env::var("NAME").unwrap_or(String::from("no_name"));
    let sensor_addr = format!("0.0.0.0:{}", sensor_port);


    // TODO confirm loop, and see if _http is needed
    // tokio::task::spawn( async move {
    //     loop {
            let responder = Responder::new().unwrap();
            let _service = responder.register(
                "_sensor._tcp".to_owned(),
                sensor_name.to_owned(),
                sensor_port.parse().unwrap(),
                &["path=/datum"]
            );
    //         // Keep the service registered for a certain duration
    //         tokio::time::sleep(Duration::from_secs(5)).await;
    //         // Loop will then re-register
    //     }
    // });


    HttpServer::new(|| {
        App::new().service(get_datum)
    })
        .bind(sensor_addr)?
        .run()
        .await
}

#[get("/datum")]
async fn get_datum() -> impl ActixResponder {
    let mut rng = rand::thread_rng();
    let data = rng.gen_range(0..100);

    HttpResponse::Ok().body(data.to_string())
}

