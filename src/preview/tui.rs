use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span},
};

use crate::{preview::SampleSize, theme::Theme};

pub fn lines(theme: &Theme, size: SampleSize) -> Vec<Line<'static>> {
    let sample = match size {
        SampleSize::Small => SMALL,
        SampleSize::Medium => MEDIUM,
        SampleSize::Large => LARGE,
    };
    let width = match size {
        SampleSize::Small => usize::from(size.dimensions().0),
        SampleSize::Medium | SampleSize::Large => sample
            .lines()
            .map(|line| Line::raw(line).width())
            .max()
            .unwrap_or_default(),
    };
    sample
        .lines()
        .map(|line| highlight(line, theme, width))
        .collect()
}

fn highlight(line: &'static str, theme: &Theme, width: usize) -> Line<'static> {
    if line.starts_with("Job monitor") {
        return Line::styled(
            format!("{line:<width$}"),
            Style::default()
                .fg(Color::from(theme.foreground))
                .bg(Color::from(theme.palette.normal[6])),
        );
    }
    if let Some(rest) = line.strip_prefix("All") {
        return Line::from(vec![Span::styled("All", Modifier::BOLD), Span::raw(rest)]);
    }
    if line.starts_with('>') {
        let style = Style::default()
            .fg(Color::from(theme.foreground))
            .bg(Color::from(theme.palette.normal[4]));
        return Line::styled(format!("{line:<width$}"), style);
    }
    if line.starts_with("CPU ") {
        let (cpu, memory) = line.split_once("MEM").expect("resource meters");
        let mut spans = bar(cpu, Color::from(theme.palette.normal[2]));
        spans.push(Span::raw("MEM"));
        spans.extend(bar(memory, Color::from(theme.palette.normal[4])));
        return Line::from(spans);
    }
    if line.starts_with("  NAME") || line.starts_with("Recent events") {
        return Line::styled(line, Modifier::BOLD);
    }
    if line.starts_with("  ") || line.starts_with("10:") {
        for (label, index) in [
            ("running", 2),
            ("queued", 3),
            ("failed", 1),
            ("done", 6),
            ("INFO", 6),
            ("WARN", 3),
            ("OK", 2),
        ] {
            if let Some(offset) = line.find(label) {
                let color = Color::from(theme.palette.normal[index]);
                let mut spans = vec![Span::raw(&line[..offset]), Span::styled(label, color)];
                spans.extend(bar(&line[offset + label.len()..], color));
                return Line::from(spans);
            }
        }
    }
    Line::raw(line)
}

fn bar(text: &'static str, color: Color) -> Vec<Span<'static>> {
    if let Some(start) = text.find('[')
        && let Some(end) = text[start..].find(']')
    {
        let end = start + end + 1;
        vec![
            Span::raw(&text[..start]),
            Span::styled(&text[start..end], color),
            Span::raw(&text[end..]),
        ]
    } else {
        vec![Span::raw(text)]
    }
}

const SMALL: &str = "\
Job monitor                 local
All  Running  Failed

  NAME     STATE     DONE
  api      running    34%
> worker   running    68%
  index    queued      0%
  backup   failed     12%
  sync     done      100%

2 running | 1 queued | 1 failed";

const MEDIUM: &str = "\
Job monitor                        local / 5 jobs
CPU [####------] 42%   MEM [###-------] 1.2G / 4G

All  Running  Queued  Failed  Done

  NAME     STATE     PROGRESS          CPU   MEMORY
  api      running   [###-------] 34%  12%   128M
> worker   running   [######----] 68%  24%   512M
  index    queued    [----------]  0%   0%    64M
  backup   failed    [#---------] 12%   0%    48M
  sync     done      [##########]100%   0%    32M

2 running | 1 queued | 1 failed | 1 done";

const LARGE: &str = "\
Job monitor                                           local / 5 jobs
CPU [########------------] 42%  MEM [######--------------] 1.2G / 4G

All  Running  Queued  Failed  Done

  NAME     STATE     PROGRESS         | Details: worker
  api      running   [###-------] 34% | State    running
> worker   running   [######----] 68% | Queue    default
  index    queued    [----------]  0% | Progress 68 / 100
  backup   failed    [#---------] 12% | CPU      24%
  sync     done      [##########]100% | Memory   512M
                                      | Elapsed  00:01:24
                                      | Task     render thumbnails

Recent events
10:42:01  INFO  worker started
10:42:02  WARN  backup target unavailable
10:42:03  OK    sync completed

2 running | 1 queued | 1 failed | 1 done";
