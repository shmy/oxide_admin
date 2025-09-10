use faktory_bg::error::RunnerError;

#[tokio::main]
async fn main() {
    let publisher = faktory_bg::queuer::Queuer::try_new("tcp://:123123@localhost:7419", "my_queue")
        .await
        .unwrap();
    publisher
        .enqueue("foobar", vec!["hello", "world"])
        .await
        .unwrap();

    let mut worker =
        faktory_bg::worker_manager::WorkerManager::new("tcp://:123123@localhost:7419", "my_queue");
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
