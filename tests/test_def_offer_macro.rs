#![feature(trait_alias)]
#![feature(impl_trait_in_assoc_type)]#![feature(associated_type_bounds)]

use generic_session_types::*;

choose_offer!(PChoose POffer {
    p_choose_1(Send<u32, Close>) OffCase1,
    p_choose_2(Recv<String, Close>) Off2
} with POfferChan);

async fn server(
    c: Chan<POffer, (), impl RawChan<R = BoxAnyRepr> + 'static>,
) -> Result<String, Error> {
    match c.offer().await? {
        POfferChan::OffCase1(c) => {
            let (number, c) = c.recv().await?;
            c.close().await?;
            Ok(format!("{}", number))
        }
        POfferChan::Off2(c) => {
            let c = c.send("okk".to_string()).await?;
            c.close().await?;
            Ok("off2".to_string())
        }
    }
}

async fn client(
    c: Chan<PChoose, (), impl RawChan<R = BoxAnyRepr> + 'static>,
) -> Result<(), Error> {
    let c = p_choose_1(c).await?;
    let c = c.send(123).await?;
    c.close().await?;
    Ok(())
}

#[tokio::test]
async fn test_def_offer_macro() {
    let (c, s) = mpsc::channel::<PChoose, BoxAnyRepr>(10);

    let h1 = tokio::spawn(client(c));
    let h2 = tokio::spawn(server(s));

    assert_eq!(h1.await.unwrap(), Ok(()));
    assert_eq!(h2.await.unwrap(), Ok(String::from("123")));

}
