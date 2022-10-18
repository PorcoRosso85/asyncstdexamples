use async_std::{
    io::{BufReader, BufWriter},
    net::TcpListner,
    task,
};
use env_logger::Env;
use futures::{prelude::*, AsyncRead, AsyncWrite};
use log::warn;
use std::error::Error;
use std::time::Duration;
use oc_http::{
    cookies::{Cookies, Cookie},
};

#[async_std::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let listner = TcpListner::bind("127.0.0.1:8080").await?;
    let _local_addr = listner.local_addr()?;
    let mut incoming = listner.incoming();
    while let Some(stream) = incoming.next().await {
        if let Ok(stream) = stream {
            task::spawn(handle_request(stream));
        }
    }
    Ok(())
}

async fn handle_request<S>(socket: S)
where
    S: AsyncRead + AsyncWrite + Clone + Unpin
{
    let mut reader = BufReader::new(socket.clone());
    let mut writer = BufWriter::new(socket);
    let mut buf = vec![0; 65536];
    let request = match oc_http::http(&mut reader, &mut buf).await {
        Ok(req) => req,
        Err(err) => {
            warn!("Error {}", err);
            return;
        }
    let mut cookies = Cookies:new(&request);
    if request.path == "/echo" && request.method == "GET" {
        get_echo(&mut writer).await;
    } else if request.path == "/echo" && request.method == "POST" {
        post_echo(&mut reader, &mut writer).await;
        if let Some(_c) = cookies.get("Who") {
            writer.write(format!("You are a fool of a took!").as_bytes()).await.unwrap();
        }
    } else {
        let mut res = oc_http::Response {
            code: 404,
            reason: "NOT FOUND",
            headers: vec!(),
        }
        cookies.add_cookie(Cookie::new("Who", "You fool!"));
        cookies.write_cookies(&mut res);
        oc_http::respond(&mut writer, res).await.unwrap();
    }
    writer.flush().await.unwrap();
}

async fn get_echo<S>(mut stream: &mut S)
where S: AsyncWrite + Unpin
{
    oc_http::respond(&mut stream, oc_http::Response {
            code: 200,
            reason: "OK",
            headers: vec![],
    }).await.unwrap();
    stream.write(b"
<html>
    <body>
        <form method=\"POST\">
            <input name=\"input\"></input>
            <input type=\"submit\"></input>
        </form>
    </body>
</html>
    ").await.unwrap();
}

async fn post_echo<W, R>(reader: &mut R, mut writer: &mut W)
where W: AsyncWrite + Unpin,
      R: AsyncRead + Unpin,
{
    oc_http::respond(&mut stream, oc_http::Response {
            code: 200,
            reason: "OK",
            headers: vec![],
    }).await.unwrap();
    let mut buf = vec![0; 10];
    while let Ok(Ok(count)) = async_std::future::timeout(Duration::from_millis(10), reader.read(&mut buf)).await {
        if count == 0 {
            break;
        }
        writer.write_all(&buf[..count]).await.unwrap();
        writer.flush().await.unwrap();
    }
}
