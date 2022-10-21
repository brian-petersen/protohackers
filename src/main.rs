mod servers;
mod util;

#[tokio::main]
async fn main() {
    let task1 = tokio::spawn(async {
        servers::smoketest::start("3000").await.unwrap();
    });

    let task2 = tokio::spawn(async {
        servers::primetime::start("3005").await.unwrap();
    });

    let task3 = tokio::spawn(async {
        servers::means_to_end::start("3010").await.unwrap();
    });

    let task4 = tokio::spawn(async {
        servers::budget_chat::start("3015").await.unwrap();
    });

    let task5 = tokio::spawn(async {
        servers::unusual_database_program::start("3020")
            .await
            .unwrap();
    });

    let task6 = tokio::spawn(async {
        servers::mob_middle::start("3025").await.unwrap();
    });

    let _ = tokio::join!(task1, task2, task3, task4, task5, task6);
}
