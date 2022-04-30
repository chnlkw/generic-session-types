use tokio::task::JoinHandle;

use crate::repr::JsonStringRepr;

use super::*;

type P1 = Send<String, Eps>;
type P1Dual = <P1 as HasDual>::Dual;

async fn run_server<C: RawChan>(server: Chan<P1Dual, C>) -> Result<String, Error>
where
    C::R: Repr<String>,
{
    let (s, c) = server.recv().await?;
    c.close().await?;
    Ok(s)
}

async fn run_client<C: RawChan>(client: Chan<P1, C>, msg: String) -> Result<(), Error>
where
    C::R: Repr<String>,
{
    let s = client.send(msg).await?;
    s.close().await?;
    Ok(())
}

#[tokio::test]
async fn test_tokio_mpsc_channel1() {
    let (client, server) = mpsc::channel::<P1, JsonStringRepr>(10);
    let msg = String::from("asdfsdfds");
    let h1: JoinHandle<Result<(), Error>> = tokio::spawn(run_client(client, msg));
    let h2: JoinHandle<Result<String, Error>> = tokio::spawn(run_server(server));
    let r0 = h1.await.unwrap();
    let r1 = h2.await.unwrap();
    assert_eq!(r0, Ok(()));
    assert_eq!(r1, Ok(String::from("asdfsdfds")));
}

type P2 = Send<String, Recv<usize, Eps>>;

#[tokio::test]
async fn test_tokio_mpsc_channel2() {
    let (client, server) = mpsc::channel::<P2, JsonStringRepr>(10);
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
    let (client, server) = mpsc::channel::<P2, DynMessage>(10);
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