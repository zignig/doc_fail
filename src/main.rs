use std::io::Bytes;

use anyhow::{Context, Result, anyhow, bail, ensure};
use iroh::{Endpoint, protocol::Router};
use iroh_blobs::{ALPN as BLOBS_ALPN, BlobsProtocol, store::mem::MemStore};
use iroh_docs::{ALPN as DOCS_ALPN, api::Doc, protocol::Docs, store::Query};
use iroh_gossip::{ALPN as GOSSIP_ALPN, net::Gossip};
use n0_future::{Stream, StreamExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // create an iroh endpoint that includes the standard discovery mechanisms
    // we've built at number0
    let endpoint = Endpoint::builder().discovery_n0().bind().await?;

    // build the blobs protocol
    let blobs = MemStore::default();

    // build the gossip protocol
    let gossip = Gossip::builder().spawn(endpoint.clone());

    // build the docs protocol
    let docs = Docs::memory()
        .spawn(endpoint.clone(), (*blobs).clone(), gossip.clone())
        .await?;

    // create a router builder, we will add the
    // protocols to this builder and then spawn
    // the router
    let builder = Router::builder(endpoint.clone());

    // setup router
    let _router = builder
        .accept(
            BLOBS_ALPN,
            BlobsProtocol::new(&blobs, endpoint.clone(), None),
        )
        .accept(GOSSIP_ALPN, gossip)
        .accept(DOCS_ALPN, docs.clone())
        .spawn();

    // do fun stuff with docs!
    let the_doc = docs.create().await?;
    let author_id = docs.author_create().await?;

    let _ = the_doc.set_bytes(author_id, "t".as_bytes(), "bork").await?;
    let _ = the_doc
        .set_bytes(author_id, "todo".as_bytes(), "bork")
        .await?;

    let _ = get_notes(&the_doc).await?;

    the_doc.set_bytes(author_id, "t", "bork2").await?;
    println!("-----------------");
    let _ = get_notes(&the_doc).await?;
    Ok(())
}

pub async fn get_notes(doc: &Doc) -> Result<()> {
    let entries = doc.get_many(Query::all()).await?;
    let mut notes = Vec::new();
    // TODO remove once entries are unpin !
    tokio::pin!(entries);
    while let Some(entry) = entries.next().await {
        notes.push(entry.unwrap());
    }
    println!("{:#?}", notes);
    Ok(())
}
