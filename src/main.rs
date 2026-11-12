use probe_schedule_client::upstream_api::probe_schedule::PSClient;

fn main() {
    let upstream = PSClient::new();
    println!("Hello, world!");
}
