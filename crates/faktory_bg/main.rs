use faktory_bg::error::RunnerError;

#[tokio::main]
async fn main() {
    let publisher =
        faktory_bg::publisher::Publisher::try_new("tcp://:123123@localhost:7419", "my_queue")
            .await
            .unwrap();
    publisher
        .publish("foobar", vec!["hello", "world"])
        .await
        .unwrap();

    let mut worker = faktory_bg::worker::Worker::new("tcp://:123123@localhost:7419", "my_queue");
    worker.register_fn("foobar", |job| async move {
        dbg!(&job);
        Ok::<(), RunnerError>(())
    });
    worker
        .run_with_signal(async move {
            std::future::pending::<()>().await;
        })
        .await
        .unwrap();
}
