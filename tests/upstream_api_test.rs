mod common;
use common::{MockUpstream, Token};
use probe_schedule_client::upstream_api::Upstream;
use tonic::Code;

struct Case {
    name: &'static str,
    input: Box<dyn Upstream>,
    want: Token,
}

#[tokio::test]
async fn test_login() {
    let cases = vec![Case {
        name: "login on startup",
        input: MockUpstream {
            token: Token::Invalid,
            return_code: Code::Unauthenticated,
        },
        want: Token::Valid,
    }];
    for tc in cases {
        let got = tc.input.login();
    }
}
