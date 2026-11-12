use probe_schedule_client::upstream_api::probe_schedule::{Opts, PSClient};

fn main() {
    let _upstream = PSClient::new(Opts {
        base_url: todo!(),
        username: todo!(),
        password: todo!(),
    });
    println!("Hello, world!");
}
