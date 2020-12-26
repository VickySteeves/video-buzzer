use futures::{FutureExt, StreamExt};
use rand::Rng;
use std::net::Ipv4Addr;
use warp::{Filter, Reply};
use warp::http::Uri;
use warp::path;
use warp::reply::with::header;

const PORT: u16 = 8000;

fn redirect_to_random_video() -> impl Reply {
    let video_id: u32 = rand::thread_rng().gen_range(1..1000000);
    let uri = Uri::builder()
        .path_and_query(format!("/video/{}", video_id))
        .build()
        .expect("Building random video URL failed");
    warp::redirect::temporary(uri)
}

#[tokio::main]
async fn main() {
    let routes =
        // Index, redirect to a video player
        path::end()
            .map(redirect_to_random_video)
        // Video player
        .or(
            warp::path!("video" / u32)
                .map(|_| include_bytes!("../video.html") as &[u8])
                .with(header("Content-Type", "text/html; charset=utf-8"))
        )
        // Buzzer join URL
        .or(
            warp::path!(u32)
                .map(|_| include_bytes!("../join.html") as &[u8])
                .with(header("Content-Type", "text/html; charset=utf-8"))
        )
        // Buzzer view
        .or(
            warp::path!("buzz" / u32 / String)
                .map(|_, _| include_bytes!("../buzzer.html") as &[u8])
                .with(header("Content-Type", "text/html; charset=utf-8"))
        )
        // API
        .or(
            warp::path("api").and(
                // Video player WebSocket
                warp::path!("host" / u32)
                    .and(warp::ws())
                    .map(|video_id, ws: warp::ws::Ws| {
                        ws.on_upgrade(|ws| {
                            // TODO: Video channel
                            let (tx, rx) = ws.split();
                            rx.forward(tx).map(|result| {
                                if let Err(e) = result {
                                    eprintln!("websocket error: {:?}", e);
                                }
                            })
                        })
                    })
                .or(
                    warp::path!("buzzer" / u32 / String)
                        .and(warp::ws())
                        .map(|video_id, player_name, ws: warp::ws::Ws| {
                            ws.on_upgrade(|ws| {
                                // TODO: Buzzer channel
                                let (tx, rx) = ws.split();
                                rx.forward(tx).map(|result| {
                                    if let Err(e) = result {
                                        eprintln!("websocket error: {:?}", e);
                                    }
                                })
                            })
                        })
                )
            )
        );

    eprintln!("Starting server on port {}", PORT);
    warp::serve(routes).run((Ipv4Addr::UNSPECIFIED, PORT)).await;
}
