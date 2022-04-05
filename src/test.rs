

use tokio::task::JoinHandle;

use crate::{repr::MyString};

use super::*;

type P1 = Send<String, Eps>;
type P2 = Send<String, Recv<usize, Eps>>;

#[tokio::test]
async fn test_tokio_mpsc_channel1() {
    let (client, server) = mpsc::session_channel::<P1, MyString>(10);
    let msg = String::from("asdfsdfds");
    let h1: JoinHandle<Result<(), Error>> = tokio::spawn(async move {
        let s = client.send(msg).await?;
        s.close().await?;
        Ok(())
    });
    let h2: JoinHandle<Result<String, Error>> = tokio::spawn(async move {
        let (s, c) = server.recv().await?;
        c.close().await?;
        Ok(s)
    });
    let r0 = h1.await.unwrap();
    let r1 = h2.await.unwrap();
    assert_eq!(r0, Ok(()));
    assert_eq!(r1, Ok(String::from("asdfsdfds")));
}

#[tokio::test]
async fn test_tokio_mpsc_channel2() {
    let (client, server) = mpsc::session_channel::<P2, MyString>(10);
    let msg = String::from("asdfsdfds");
    let h1: JoinHandle<Result<usize, Error>> = tokio::spawn(async move {
        let s = client.send(msg).await?;
        let (r, s) = s.recv().await?;
        s.close().await?;
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

#[tokio::test]
async fn test_tokio_mpsc_channel2_dyn_message() {
    let (client, server) = mpsc::session_channel::<P2, DynMessage>(10);
    let msg = String::from("asdfsdfds");
    let h1: JoinHandle<Result<usize, Error>> = tokio::spawn(async move {
        let s = client.send(msg).await?;
        let (r, s) = s.recv().await?;
        s.close().await?;
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

