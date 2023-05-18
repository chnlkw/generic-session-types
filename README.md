# generic-session-types

Generic session types in Rust for async send recv channel

# Prerequisites

* [Cargo](https://rustup.rs/)
* Rust nightly
```sh
rustup default nightly
```

# Usage

Add dependency to `Cargo.toml`
```toml
[dependencies]
generic-session-types = "0.1.1"
```

# Example

```rust

use generic_session_types::*;

type P2 = Send<String, Recv<usize, Close>>;

#[tokio::test]
async fn test_tokio_mpsc_channel2_dyn_message() {
    let (client, server) = mpsc::channel::<P2, BoxAnyRepr>(10);
    let msg = String::from("asdfsdfds");
    let h1: JoinHandle<Result<usize, Error>> = tokio::spawn(async move {
        send!(client, msg);
        recv!(client, r);
        close!(client);
        Ok(r)
    });
    let h2: JoinHandle<Result<(), Error>> = tokio::spawn(async move {
        let (s, c) = server.recv().await?;
        let c = c.send(s.len()).await?;
        c.close().await?;
        Ok(())
    });
    let r0 = h1.await.unwrap();
    let r1 = h2.await.unwrap();
    assert_eq!(r0, Ok(9));
    assert_eq!(r1, Ok(()));
}
```

* See more cases in [tests](tests/)


# References

* [Munksgaard session-type](https://github.com/Munksgaard/session-types)
* [async-session-types](https://github.com/aakoshh/async-session-types-rs)
* [Session Types for Rust](http://munksgaard.me/papers/laumann-munksgaard-larsen.pdf)