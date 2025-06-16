use tracing::log::{info, Level, LevelFilter, Metadata, Record};
use std::io::Write;
use tokio::io::{AsyncWriteExt};
use tokio::net::TcpStream;

// // A simple logger implementation that sends logs to Vector
// struct VectorLogger {
//     stream: TcpStream,
// }

// impl VectorLogger {
//     async fn new(host: &str, port: u16) -> Result<Self, std::io::Error> {
//         let stream = TcpStream::connect(format!("{}:{}", host, port)).await?;
//         Ok(VectorLogger { stream })
//     }
// }

// impl log::Log for VectorLogger {
//     fn enabled(&self, _metadata: &Metadata) -> bool {
//         true
//     }

//     fn log(&self, record: &Record) {
//         let _ = self.log_async(record).await;
//     }

//     fn flush(&self) {}
// }

// impl VectorLogger {
//     async fn log_async(&self, record: &Record) -> Result<(), std::io::Error> {
//         let log_event = format!(
//             "{{\"level\":\"{}\", \"message\":\"{}\", \"module\":\"{}\", \"file\":\"{}\", \"line\":{}}}\n",
//             record.level(),
//             record.args(),
//             record.location().module_path(),
//             record.location().file(),
//             record.location().line()
//         );

//         self.stream.write_all(log_event.as_bytes()).await?;
//         Ok(())
//     }
// }

