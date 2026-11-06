use ratatui::{
    style::Color,
    text::{Line, Span},
};

use crate::{preview::SampleSize, theme::Theme};

pub fn lines(theme: &Theme, size: SampleSize) -> Vec<Line<'static>> {
    let sample = match size {
        SampleSize::Small => SMALL,
        SampleSize::Medium => MEDIUM,
        SampleSize::Large => LARGE,
    };
    sample.lines().map(|line| highlight(line, theme)).collect()
}

fn highlight(line: &'static str, theme: &Theme) -> Line<'static> {
    let Some((time, rest)) = line
        .split_once(' ')
        .filter(|(time, _)| time.chars().next().is_some_and(|c| c.is_ascii_digit()))
    else {
        return Line::raw(line);
    };
    let (level, message) = rest.split_once(' ').expect("log level has a message");
    let color = match level {
        "DEBUG" => theme.palette.normal[5],
        "INFO" => theme.palette.normal[6],
        "OK" => theme.palette.normal[2],
        "WARN" => theme.palette.normal[3],
        "ERROR" => theme.palette.normal[1],
        _ => unreachable!("bundled log level"),
    };
    Line::from(vec![
        Span::styled(time, Color::from(theme.palette.bright[0])),
        Span::raw(" "),
        Span::styled(level, Color::from(color)),
        Span::raw(" "),
        Span::raw(message),
    ])
}

const SMALL: &str = "\
10:42:00 INFO  worker started
10:42:00 DEBUG config loaded
10:42:01 INFO  listening on :8080
10:42:03 INFO  GET /items/17
10:42:03 WARN  cache miss
10:42:03 ERROR store timeout
10:42:03 INFO  retry 1/3
10:42:03 OK    200 /items/17

Requests: 1  Retried: 1  Failed: 0";

const MEDIUM: &str = "\
10:42:00 INFO  app    worker started
10:42:00 DEBUG cfg    loaded config/default.toml
10:42:01 INFO  http   listening on 127.0.0.1:8080

10:42:03 INFO  http   r17 GET /items/17
10:42:03 DEBUG cache  r17 lookup key=item:17
10:42:03 WARN  cache  r17 miss; reading store
10:42:03 ERROR store  r17 timeout after 100ms
                      cause: connection timed out
10:42:03 INFO  store  r17 retry 1/3
10:42:03 OK    http   r17 200 OK (240ms)

Requests: 1  Retried: 1  Failed: 0";

const LARGE: &str = "\
10:42:00.000 INFO  app    worker started version=0.3.0
10:42:00.010 DEBUG cfg    loaded path=config/default.toml
10:42:01.000 INFO  http   listening address=127.0.0.1:8080

10:42:03.210 INFO  http   request=r17 method=GET path=/items/17
10:42:03.220 DEBUG cache  request=r17 key=item:17
10:42:03.220 WARN  cache  request=r17 result=miss
10:42:03.320 ERROR store  request=r17 timeout=100ms
                          caused by: connection timed out
                          endpoint: store.local:9000
10:42:03.420 INFO  store  request=r17 retry=1/3 backoff=100ms
10:42:03.450 OK    http   request=r17 status=200 elapsed=240ms

10:42:04.100 INFO  http   request=r18 method=GET path=/health
10:42:04.102 OK    http   request=r18 status=200 elapsed=2ms

Requests: 2  Retried: 1  Failed: 0";
