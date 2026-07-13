use std::io::{self, BufRead, Write};

use kickhatsnare_core::Core;
use kickhatsnare_protocol::{ErrorCode, PROTOCOL_VERSION, Request, Response};

use crate::api;

pub fn serve(reader: impl BufRead, mut writer: impl Write, core: &mut Core) -> io::Result<()> {
    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }

        let response = handle_line(&line, core);
        serde_json::to_writer(&mut writer, &response).map_err(io::Error::other)?;
        writer.write_all(b"\n")?;
        writer.flush()?;
    }

    Ok(())
}

fn handle_line(line: &str, core: &mut Core) -> Response {
    let request = match serde_json::from_str::<Request>(line) {
        Ok(request) => request,
        Err(error) => return Response::error(0, ErrorCode::InvalidRequest, error.to_string()),
    };

    if request.protocol_version != PROTOCOL_VERSION {
        return Response::error(
            request.id,
            ErrorCode::ProtocolVersionMismatch,
            format!(
                "expected protocol version {PROTOCOL_VERSION}, received {}",
                request.protocol_version
            ),
        );
    }

    match api::dispatch(&request.method, &request.params, core) {
        Ok(result) => Response::success(request.id, result),
        Err(error) => Response::error(request.id, error.code, error.message),
    }
}

#[cfg(test)]
mod tests {
    use kickhatsnare_core::Core;
    use serde_json::Value;

    use super::serve;

    #[test]
    fn handles_a_ping_request() {
        let input = b"{\"protocolVersion\":7,\"id\":7,\"method\":\"system.ping\",\"params\":{}}\n";
        let mut output = Vec::new();

        serve(input.as_slice(), &mut output, &mut Core::new()).expect("request should succeed");

        assert_eq!(
            String::from_utf8(output).expect("response should be UTF-8"),
            "{\"protocolVersion\":7,\"id\":7,\"result\":\"ready\"}\n"
        );
    }

    #[test]
    fn returns_the_initial_workspace_snapshot() {
        let input =
            b"{\"protocolVersion\":7,\"id\":10,\"method\":\"workspace.get\",\"params\":{}}\n";
        let mut output = Vec::new();

        serve(input.as_slice(), &mut output, &mut Core::new()).expect("request should succeed");
        let response: Value = serde_json::from_slice(&output).expect("response should be JSON");

        assert_eq!(response["id"], 10);
        assert_eq!(response["result"]["name"], "Untitled");
        assert_eq!(
            response["result"]["timeline"]["tracks"]
                .as_array()
                .map(Vec::len),
            Some(30)
        );
        assert_eq!(response["result"]["timeline"]["tracks"][0]["id"], "track-1");
        assert_eq!(
            response["result"]["timeline"]["tracks"][29]["id"],
            "track-30"
        );
        assert_eq!(response["result"]["isDirty"], false);
    }

    #[test]
    fn returns_the_initial_library_snapshot() {
        let input = b"{\"protocolVersion\":7,\"id\":11,\"method\":\"library.get\",\"params\":{}}\n";
        let mut output = Vec::new();

        serve(input.as_slice(), &mut output, &mut Core::new()).expect("request should succeed");

        assert_eq!(
            String::from_utf8(output).expect("response should be UTF-8"),
            "{\"protocolVersion\":7,\"id\":11,\"result\":{\"pinnedFolders\":[]}}\n"
        );
    }

    #[test]
    fn routes_timeline_mutations_and_returns_the_updated_project() {
        let input = b"{\"protocolVersion\":7,\"id\":12,\"method\":\"workspace.saveTimelineTrack\",\"params\":{\"id\":null,\"name\":\"Drums\",\"isMuted\":false,\"isSoloed\":false}}\n";
        let mut output = Vec::new();

        serve(input.as_slice(), &mut output, &mut Core::new()).expect("request should succeed");
        let response: Value = serde_json::from_slice(&output).expect("response should be JSON");

        assert_eq!(response["id"], 12);
        assert_eq!(
            response["result"]["timeline"]["tracks"][30]["id"],
            "track-31"
        );
        assert_eq!(
            response["result"]["timeline"]["tracks"][30]["name"],
            "Drums"
        );
        assert_eq!(response["result"]["isDirty"], true);
    }

    #[test]
    fn routes_feature_domains_independently() {
        let input =
            b"{\"protocolVersion\":7,\"id\":8,\"method\":\"audio.unknown\",\"params\":{}}\n";
        let mut output = Vec::new();

        serve(input.as_slice(), &mut output, &mut Core::new()).expect("request should succeed");

        assert_eq!(
            String::from_utf8(output).expect("response should be UTF-8"),
            "{\"protocolVersion\":7,\"id\":8,\"error\":{\"code\":\"METHOD_NOT_FOUND\",\"message\":\"unknown audio method: unknown\"}}\n"
        );
    }

    #[test]
    fn rejects_a_mismatched_protocol_version() {
        let input = b"{\"protocolVersion\":8,\"id\":9,\"method\":\"system.ping\",\"params\":{}}\n";
        let mut output = Vec::new();

        serve(input.as_slice(), &mut output, &mut Core::new()).expect("request should succeed");

        assert_eq!(
            String::from_utf8(output).expect("response should be UTF-8"),
            "{\"protocolVersion\":7,\"id\":9,\"error\":{\"code\":\"PROTOCOL_VERSION_MISMATCH\",\"message\":\"expected protocol version 7, received 8\"}}\n"
        );
    }
}
