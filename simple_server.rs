use std::{
    error::Error
};
use log::{warn};
use env_logger::Env;
use async_std::{
    task,
    io::{
        BufReader,
        BufWriter,
    },
    net::TcpListner,
};
use futures::{
    prelude::*,
    AsyncRead,
    AsyncWrite,
}

#[async_std::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let listner = TcpListner::bind("127.0.0.1:8080").await?;
    let mut incoming = listner.incoming();
    while let Some(stream) = incoming.next().await {
        if let Ok(stream) = stream {
            task::spawn(handle_request(stream));
        }
    }
    Ok(())
}

async fn handle_request<S>(stream: S)
where S: AsyncRead + AsyncWrite + Clone + Unpin
{
    let mut reader = BufReader::new(stream.clone());
    let mut writer = BufWriter::new(stream);
    let mut buf = vec![0; 65536];
    match oc_http::http(&mut reader, &mut buf).await {
        Ok(req) => req,
        Err(err) => {
            warn!("Error {}", err);
            return;
        },
    };
    oc_http::respond(&mut writer, oc_http::Response{
        code: 200,
        reason: "OK",
        headers: vec!(),
    }).await.unwrap();
    writer.write(b"
<html>
    <body>
        <h1>Hello World</h1>
    </body>
</html>
    ").await.unwrap();
    writer.flush().await.unwrap();
}

