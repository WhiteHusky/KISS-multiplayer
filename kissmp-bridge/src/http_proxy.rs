use percent_encoding::percent_decode_str;
use std::net::Ipv4Addr;

pub fn spawn_http_proxy(mut discord_tx: tokio::sync::mpsc::Sender<crate::DiscordState>) {
        // Master server proxy
    std::thread::spawn( || async move {
        let server = tiny_http::Server::http("0.0.0.0:3693").unwrap();
        loop{
            for request in server.incoming_requests() {
                let addr = request.remote_addr();
                if addr.ip() != Ipv4Addr::new(127, 0, 0, 1) {
                    continue;
                }
                let mut url = request.url().to_string();
                url.remove(0);
                if url == "check" {
                    let response = tiny_http::Response::from_string("ok");
                    request.respond(response).unwrap();
                    continue;
                }
                #[cfg(feature = "discord-rpc-client")]
                if url.starts_with("rich_presence") {
                    let server_name_encoded = url.replace("rich_presence/", "");
                    let data = percent_decode_str(&server_name_encoded)
                        .decode_utf8_lossy()
                        .into_owned();
                    let server_name = {
                        if data != "none" {
                            Some(data)
                        } else {
                            None
                        }
                    };
                    let state = crate::DiscordState { server_name };
                    discord_tx.send(state).await.unwrap();
                    let response = tiny_http::Response::from_string("ok");
                    request.respond(response).unwrap();
                    continue;
                }
                if let Ok(response) = reqwest::get(&url).await {
                    if let Ok(text) = response.text().await {
                        let response = tiny_http::Response::from_string(text);
                        request.respond(response).unwrap();
                    }
                }
            }
        }
    });
}
