use criterion::{criterion_group, criterion_main, Criterion};

use bson_2_2_0::RawDocumentBuf;
use futures::TryStreamExt;
use mongodb::{
    bson::{doc, oid::ObjectId, Document},
    Client,
};

pub fn rawbson_bench(c: &mut Criterion) {
    // begin setup

    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();

    let doc = doc! {
        "a": true,
        "b": "hello",
        "c": {
            "world": 1_i32,
            "ok": 2_i32,
            "other_key": 5.5,
        },
        "oid": ObjectId::new(),
        "array": [1_i32, 2_i32, 3_i32],
    };

    let (coll_2_1_0, coll_2_2_0) = rt.block_on(async {
        let client = Client::with_uri_str("mongodb://localhost:27017")
            .await
            .unwrap();
        let coll = client.database("bench").collection::<Document>("coll");
        coll.drop(None).await.unwrap();

        let docs = vec![&doc; 10_000];
        coll.insert_many(docs, None).await.unwrap();

        let client_2_2_0 = mongodb_2_2_0::Client::with_uri_str("mongodb://localhost:27017")
            .await
            .unwrap();
        let coll_2_2_0 = client_2_2_0
            .database("bench")
            .collection::<Document>("coll");

        (coll, coll_2_2_0)
    });

    let mut group = c.benchmark_group("bench");

    group.bench_function("2.1.0 find 10k", |b| {
        b.to_async(&rt).iter(|| async {
            let _ = coll_2_1_0
                .find(None, None)
                .await
                .unwrap()
                .try_collect::<Vec<Document>>()
                .await
                .unwrap();
        })
    });

    group.bench_function("2.2.0 find 10k", |b| {
        b.to_async(&rt).iter(|| async {
            let _ = coll_2_2_0
                .find(None, None)
                .await
                .unwrap()
                .try_collect::<Vec<Document>>()
                .await
                .unwrap();
        })
    });

    group.bench_function("2.1.0 find Document -> json", |b| {
        b.to_async(&rt).iter(|| async {
            let arr = coll_2_1_0
                .find(None, None)
                .await
                .unwrap()
                .try_collect::<Vec<Document>>()
                .await
                .unwrap();

            let _ = serde_json::to_string_pretty(&arr).unwrap();
        })
    });

    group.bench_function("2.2.0 find RawDocumentBuf -> json", |b| {
        b.to_async(&rt).iter(|| async {
            let arr = coll_2_2_0
                .clone_with_type::<RawDocumentBuf>()
                .find(None, None)
                .await
                .unwrap()
                .try_collect::<Vec<RawDocumentBuf>>()
                .await
                .unwrap();

            let _ = serde_json::to_string_pretty(&arr).unwrap();
        })
    });

    group.finish();
}

criterion_group!(benches, rawbson_bench);
criterion_main!(benches);
