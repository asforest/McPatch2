use std::convert::Infallible;
use std::net::SocketAddr;

use dav_server::fakels::FakeLs;
use dav_server::memfs::MemFs;
use dav_server::DavHandler;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper_util::rt::TokioIo;
use tokio::net::TcpListener;

use crate::AppContext;

pub fn do_dav(_ctx: &AppContext) -> i32 {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();

    runtime.block_on(async {
        // let dir = "/tmp";
        let addr: SocketAddr = ([127, 0, 0, 1], 4918).into();
    
        let dav_server = DavHandler::builder()
            .filesystem(MemFs::new())
            .locksystem(FakeLs::new())
            .build_handler();

        let listener = TcpListener::bind(addr).await.unwrap();

        println!("Listening {addr}");
    
        // We start a loop to continuously accept incoming connections
        loop {
            let (stream, _) = listener.accept().await.unwrap();
            let dav_server = dav_server.clone();
    
            // Use an adapter to access something implementing `tokio::io` traits as if they implement
            // `hyper::rt` IO traits.
            let io = TokioIo::new(stream);
    
            // Spawn a tokio task to serve multiple connections concurrently
            tokio::task::spawn(async move {
                // Finally, we bind the incoming connection to our `hello` service
                if let Err(err) = http1::Builder::new()
                    // `service_fn` converts our function in a `Service`
                    .serve_connection(
                        io,
                        service_fn({
                            move |req| {
                                let dav_server = dav_server.clone();
                                async move { Ok::<_, Infallible>(dav_server.handle(req).await) }
                            }
                        }),
                    )
                    .await
                {
                    eprintln!("Failed serving: {err:?}");
                }
            });
        }
    });

    0
}