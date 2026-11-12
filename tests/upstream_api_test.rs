mod common;
use common::{KnownUpstream, MockUpstream, Token};
use probe_schedule_client::upstream_api::Upstream;
use tonic::Code;

struct Case {
    upstream: KnownUpstream,
    name: &'static str,
    want: Token,
}

#[tokio::test]
async fn test_login() {
    let cases = vec![Case {
        name: "login on startup",
        upstream: KnownUpstream::Mock(MockUpstream {
            token: Token::Invalid,
            return_code: Code::Unauthenticated,
        }),
        want: Token::Valid,
    }];
    for tc in cases {
        let got = tc.upstream.login();
    }
}
