use lunatic::process::StartProcess;
use lunatic::{abstract_process, Tag};
use lunatic::{process::ProcessRef, Process};
use mysql::prelude::*;
use mysql::*;
use serde::{Deserialize, Serialize};
use submillisecond::{router, Application, Json};

struct DbHandler {
    pool: Pool,
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
struct Log {
    id: i32,
    content: String,
}

#[abstract_process]
impl DbHandler {
    #[init]
    fn init(_: ProcessRef<Self>, url: String) -> Self {
        let pool = Pool::new(url.as_str()).expect("should create pool");
        let mut conn = pool.get_conn().unwrap();
        conn.query_drop(
            r"CREATE TABLE IF NOT EXISTS logs (
                log_id int not null,
                content text
            )",
        )
        .unwrap();
        Self { pool }
    }

    #[terminate]
    fn terminate(self) {
        println!("Shutdown process");
    }

    #[handle_link_trapped]
    fn handle_link_trapped(&self, _tag: Tag) {
        println!("Link trapped");
    }

    // we assume the table already exists, just an example of how you could write data with a "request"
    #[handle_request]
    fn write_log(&mut self, log: Log) -> std::result::Result<(), String> {
        let mut conn = self.pool.get_conn().expect("should get connection");
        conn.exec_drop(
            r"INSERT INTO logs (log_id, content) VALUES (?, ?)",
            (log.id, log.content),
        )
        .expect("should run exec");
        Ok(())
    }
}

fn index(Json(log): Json<Log>) -> &'static str {
    let db = ProcessRef::<DbHandler>::lookup("DB_HANDLER").expect("should have found db process");
    db.write_log(log).expect("should have written log to db");
    "Hello :)"
}

fn main() -> std::io::Result<()> {
    // register process with name "DB_HANDLER"
    let _db = DbHandler::start(
        "mysql://root:@localhost:3306/db_name".to_string(),
        Some("DB_HANDLER"),
    );
    Application::new(router! {
        POST "/" => index
    })
    .serve("0.0.0.0:3000")
}
