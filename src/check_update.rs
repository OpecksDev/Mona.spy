use reqwest;
use std::thread;
use std::time::Duration;

pub async fn call_every_hour(addr: String, paths: Vec<&'static str>) {
  let paths: Vec<_> = paths
    .into_iter()
    .map(|path| "http://".to_owned() + addr.as_str() + path)
    .collect();

  thread::sleep(Duration::from_secs(5));
  loop {
    for path in &paths {
      println!("Starting Hourly Call");
      let client = reqwest::Client::new();
      let resp = client.get(path).send().await;
      match resp {
        Ok(_) => {}
        Err(err) => println!("Error during hourly call: {:?}", err),
      }
    }
    thread::sleep(Duration::from_secs(3600));
  }
}
