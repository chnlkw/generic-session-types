#![feature(trait_alias)]

use tokio::task::JoinHandle;

use generic_session_types::*;

type P1 = Send<String, Close>;
type P1Dual = <P1 as HasDual>::Dual;

trait Reprs = Repr<String> + Repr<u32>; // this is a trait alias

async fn run_server<C: RawChan>(server: Chan<P1Dual, C>) -> Result<String, Error>
where
    C::R: Reprs,
{
    let (s, c) = server.recv().await?;
    c.close().await?;
    Ok(s)
}

async fn run_client<C: RawChan>(client: Chan<P1, C>, msg: String) -> Result<(), Error>
where
    C::R: Reprs,
{
    let s = client.send(msg).await?;
    s.close().await?;
    Ok(())
}

#[tokio::test]
async fn test_tokio_mpsc_channel1() {
    let (client, server) = mpsc::channel::<P1, BoxAnyRepr>(10);
    let msg = String::from("asdfsdfds");
    let h1: JoinHandle<Result<(), Error>> = tokio::spawn(run_client(client, msg));
    let h2: JoinHandle<Result<String, Error>> = tokio::spawn(run_server(server));
    let r0 = h1.await.unwrap();
    let r1 = h2.await.unwrap();
    assert_eq!(r0, Ok(()));
    assert_eq!(r1, Ok(String::from("asdfsdfds")));
}

type P2 = Send<String, Recv<usize, Close>>;

#[tokio::test]
async fn test_tokio_mpsc_channel2() {
    let (client, server) = mpsc::channel::<P2, BoxAnyRepr>(10);
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
    let (client, server) = mpsc::channel::<P2, BoxAnyRepr>(10);
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

type P3 = Choose<Send<u32, Close>, Recv<String, Close>>;

#[tokio::test]
async fn test_offer_choice() {
    let (client, server) = mpsc::channel::<P3, BoxAnyRepr>(10);
    let h1: JoinHandle<Result<(), Error>> = tokio::spawn(async move {
        let s = client.left().await?;
        let s = s.send(123).await?;
        s.close().await?;
        Ok(())
    });
    let h2: JoinHandle<Result<Vec<u32>, Error>> = tokio::spawn(async move {
        let ret = match server.offer().await? {
            Branch::Left(l) => {
                let (data, s) = l.recv().await?;
                s.close().await?;
                vec![data]
            }
            Branch::Right(r) => {
                let s = r.send("23".to_string()).await?;
                s.close().await?;
                vec![]
            }
        };
        Ok(ret)
    });
    let r0 = h1.await.unwrap();
    let r1 = h2.await.unwrap();
    assert_eq!(r0, Ok(()));
    assert_eq!(r1, Ok(vec![123]));
}
