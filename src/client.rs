//use std::collections::HashMap;
use futures::{FutureExt, StreamExt};
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use warp::ws::{Message, WebSocket};

// When you push messages to a client's sender, an asynchronous coroutine
// forwards them to the external (out of process) client via websocket.
type Sink = mpsc::UnboundedSender<Result<Message, warp::Error>>;

// TODO: Support simultaneous clients.
#[derive(Clone, Debug)]
pub struct ClientPointer {
    lock: Arc<RwLock<Option<Sink>>>,
}

impl ClientPointer {
    // Creates a pointer not connected to any client.  Call `connect` to enable
    // the send method.
    pub fn new() -> ClientPointer {
        ClientPointer {
            lock: Arc::new(RwLock::new(None)),
        }
    }

    // Assigns a message sink (i.e., a write-only buffer view) to this client,
    // and copies messages (from the read-only end of that buffer) to the
    // specified WebSocket until the WebSocket is no longer readable.
    //
    // TODO: Normalize logging.
    // TODO: Factor out `connect_outgoing` and `connect_incoming` functions.
    pub async fn connect(self, ws: WebSocket) {
        let (ws_sink, mut ws_source) = ws.split();
        let (buf_sink, buf_source) = mpsc::unbounded_channel();

        // Handle outgoing messages.
        //
        // Launch an asynchronous task to forward all messages from the
        // channel to the websocket.  The channel is only a server-side buffer,
        // whereas the websocket is the network connection to a client
        // application.
        //
        // Before we can spawn the task, we must "enter the runtime context,"
        // as the Tokio [docs][] put it, or else Tokio panics:
        //
        // > thread 'tokio-runtime-worker' panicked at 'must be called from the
        // > context of Tokio runtime configured with either `basic_scheduler`
        // > or `threaded_scheduler`'
        //
        // This frankly feels silly to me, as we're already being blocked on by
        // a Tokio runtime created in `crate::main`.  Moreover, "entering the
        // runtime context" in `main` instead of here does not fix the problem.
        //
        // If we downgrade to Tokio 0.2, we no longer have to do this; so the
        // silliness may be an artifact of Tokio 0.2/0.3 incompatibility that
        // can go away once Warp (from which we got the WebSocket) upgrades to
        // 0.3.  The WebSocket itself seems to be from a library called
        // Tungstenite, whose authors also provide the Tokio/Tungstenite
        // [bindings][] used by Warp, so it may be some time before any version
        // upgrades bubble up through the stack to Whim.
        //
        // [docs]: https://docs.rs/tokio/0.3.4/tokio/runtime/struct.Runtime.html#method.enter
        // [bindings]: https://github.com/snapview/tokio-tungstenite
        let rt = tokio::runtime::Runtime::new().unwrap();
        let _guard = rt.enter();
        tokio::spawn(buf_source.forward(ws_sink).map(|result| {
            if let Err(e) = result {
                eprintln!("error sending websocket msg: {}", e);
            }
        }));
        *self.lock.write().await = Some(buf_sink);
        println!("connected");

        // Handle incoming messages until the connection drops (e.g., because
        // the client closes it), then drop the channel.
        //
        // TODO: Move the clean-up code to the destructor of a local sentry, so
        // it gets called even if we have to bail early for some reason.  The
        // you can replace the match/break logic with a `?`.
        while let Some(result) = ws_source.next().await {
            match result {
                Ok(message) => {
                    println!("received: {:?}", message);
                }
                Err(err) => {
                    eprintln!("error receiving ws message: {}", err);
                    break;
                }
            };
        }
        *self.lock.write().await = None;
        println!("disconnected");
    }

    pub async fn send(&self, message: String) {
        if let Some(ref sink) = *self.lock.write().await {
            if let Err(err) = sink.send(Ok(Message::text(message))) {
                eprintln!("error buffering outgoing message: {}", err);
            }
        } else {
            eprintln!("warning: write to null client pointer");
        }
    }
}

// TODO: This name is too long.  The LogRocket guy calls his `Clients`, but
// when I read his code, I have to remind myself of the type every time it's
// accessed anyway.  The "right" solution is probably to wrap this composition
// of five types (pointer, lock, map, key, and value) in a single type that
// supports simple operations, but what would such a type be called?
//
// Considerations:
//
// * The referential nature of the type is important.  When somebody clones
//   this, it must be obvious that they are cloning a pointer, not a map.
//
// * The lock takes care of itself:  If somebody forgets to lock, the code
//   won't compile, and the error message will mention the lock type.
// type ClientMapPointer = Arc<RwLock<HashMap<String, Client>>>;
