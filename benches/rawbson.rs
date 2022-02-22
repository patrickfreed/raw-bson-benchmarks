use criterion::{criterion_group, criterion_main, Criterion};

use bson_2_2_0::RawDocumentBuf;
use futures::TryStreamExt;
use mongodb::{
    bson::{doc, oid::ObjectId, spec::BinarySubtype, Binary, Document},
    Client,
};

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
struct LargeDoc {
    a: String,
    b: String,
    c: String,
    d: String,
    #[serde(with = "serde_bytes")]
    e: Vec<u8>,
}

#[derive(Debug, Deserialize)]
struct LargeDocRef<'a> {
    #[serde(borrow)]
    a: &'a str,
    #[serde(borrow)]
    b: &'a str,
    #[serde(borrow)]
    c: &'a str,
    #[serde(borrow)]
    d: &'a str,
    #[serde(with = "serde_bytes")]
    e: &'a [u8],
}

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
            "other_key": 5.5_f64,
        },
        "oid": ObjectId::new(),
        "array": [1_i32, 2_i32, 3_i32],
    };

    let large_doc = doc! {
        "a": "Lorem ipsum dolor sit amet, consectetur adipiscing elit. Donec mattis sodales lacus, vitae mattis arcu euismod in. Cras rutrum semper dolor eu
egestas. Duis molestie nec sem vitae rhoncus. Maecenas tincidunt sagittis lectus id condimentum. Aliquam erat volutpat. Quisque commodo
vulputate arcu, ac pharetra risus rhoncus nec. Suspendisse eget justo ac dolor tempus eleifend vel nec sapien. Proin erat elit, placerat at eros
sed, blandit aliquet dolor. Nam vulputate orci non ultricies fringilla. Aliquam mattis venenatis massa, id condimentum nunc interdum luctus.
Praesent varius tortor eget dolor porta, at condimentum dui iaculis. Pellentesque tincidunt massa mauris, et consequat turpis mattis et. Nulla
quis ullamcorper elit, nec fermentum dolor. Pellentesque non viverra mauris. Donec vitae ante rhoncus, sagittis augue nec, accumsan dolor.
Donec sit amet laoreet nisi.",
        "b": "Ut vel mi nec tellus posuere tempor ut a massa. Pellentesque elementum tristique lacus, in pretium ligula porta a. Cras a nulla dapibus,
consectetur lectus at, auctor nunc. In mi ex, condimentum vel nunc eu, congue faucibus enim. Donec ac convallis velit. In blandit est quis lectus
auctor, ut fringilla nibh fringilla. Integer rhoncus nibh id consequat posuere. Nam pulvinar nulla metus, ut ullamcorper enim aliquet eu. Aenean
est risus, varius varius nunc non, auctor fermentum tortor. In hac habitasse platea dictumst. Aenean eu faucibus velit. Donec non diam a eros
vulputate scelerisque eu at nisi. Proin purus tortor, tincidunt eget risus a, vehicula aliquam sem. Vivamus scelerisque sem vel egestas luctus.
Proin tincidunt magna ac nisi iaculis tincidunt. Nulla congue libero a sem elementum, vitae euismod nisl porttitor.",
        "c": "Nulla id vehicula tellus, eget iaculis nibh. Donec eu neque sit amet tellus ornare mattis sed gravida mauris. Mauris finibus diam nec purus
dapibus, quis interdum mauris tristique. Donec feugiat turpis eleifend enim aliquam, non cursus dolor aliquet. Curabitur mattis lacinia neque.
Nullam tincidunt sem vel neque tincidunt, nec luctus massa tempor. Sed euismod aliquet mauris hendrerit consectetur.",
        "d": "Pellentesque ut eros nec nisl luctus congue. Fusce ac nisi pulvinar, cursus magna vitae, consequat eros. Nullam eleifend at sem non dapibus.
Duis nec vehicula orci, vel tincidunt arcu. Maecenas at metus ac neque lacinia euismod. Interdum et malesuada fames ac ante ipsum primis in
faucibus. Pellentesque habitant morbi tristique senectus et netus et malesuada fames ac turpis egestas. Quisque varius vel odio eu efficitur.
Donec eget mattis lorem.",
        "e": Binary {
            bytes: vec![1u8; 1024*1024],
            subtype: BinarySubtype::Generic
        }
    };

    let (coll_2_1_0, coll_2_2_0, large_coll_2_2_0) = rt.block_on(async {
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

        let large_coll_2_2_0 = client_2_2_0
            .database("bench")
            .collection::<Document>("large_doc");
        large_coll_2_2_0.drop(None).await.unwrap();
        let docs = vec![&large_doc; 100];
        large_coll_2_2_0.insert_many(docs, None).await.unwrap();

        (coll, coll_2_2_0, large_coll_2_2_0)
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

    group.bench_function("2.2.0 find 100 large_doc owned serde", |b| {
        b.to_async(&rt).iter(|| async {
            let mut cursor = large_coll_2_2_0
                .clone_with_type::<LargeDoc>()
                .find(None, None)
                .await
                .unwrap();
            while let Some(d) = cursor.try_next().await.unwrap() {}
        })
    });

    group.bench_function("2.2.0 find 100 large_doc zero-copy serde", |b| {
        b.to_async(&rt).iter(|| async {
            let mut cursor = large_coll_2_2_0
                .clone_with_type::<LargeDocRef>()
                .find(None, None)
                .await
                .unwrap();
            while cursor.advance().await.unwrap() {
                cursor.deserialize_current().unwrap();
            }
        })
    });

    group.finish();
}

criterion_group!(benches, rawbson_bench);
criterion_main!(benches);
