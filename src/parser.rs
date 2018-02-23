#![allow(dead_code)]

use models::types::MessageFlag;
use collector::{Collector, RawMessage};
use errors::{Result, ErrorKind};

use chrono::prelude::*;
use chrono::Duration;
use nom::{IResult, not_line_ending, digit, space, line_ending};

use std::io::BufRead;
use std::str::FromStr;
use std::ops::Add;

pub trait LogParser {
    fn parse<T: BufRead + ? Sized>(collector: &mut Collector, input: T) -> Result<()> where T: Sized;
}

pub struct ChattyParser {}

#[derive(Debug, Clone, PartialEq)]
struct MessageTimestamp {
    pub time: NaiveTime,
    pub date: Option<NaiveDate>
}

impl MessageTimestamp {
    fn into_datetime(self, prev_datetime: DateTime<FixedOffset>) -> Result<DateTime<FixedOffset>> {
        if let Some(date) = self.date {
            Ok(prev_datetime.offset()
                .from_local_datetime(&date.and_time(self.time))
                .single()
                .ok_or(ErrorKind::TimestampError)?)
        } else {
            let as_today = prev_datetime.date()
                .and_time(self.time)
                .ok_or(ErrorKind::TimestampError)?;
            if as_today < prev_datetime {
                Ok(as_today)
            } else {
                Ok(prev_datetime.date()
                    .add(Duration::days(1))
                    .and_time(self.time)
                    .ok_or(ErrorKind::TimestampError)?)
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
enum Line {
    BeginLog(DateTime<FixedOffset>),
    EndLog(DateTime<FixedOffset>),
    Message {
        time: MessageTimestamp,
        message: String,
        sender: MessageSender
    },
    SystemMessage {
        time: MessageTimestamp,
        message: String,
    },
    JoinedChannel {
        time: MessageTimestamp,
        channel: String,
    },
    Separator,
    Other(String)
}

#[derive(Debug, Clone, PartialEq)]
struct MessageSender {
    pub name: String,
    pub modifiers: Vec<MessageFlag>
}

fn parse_date_time(data: &str) -> Result<DateTime<FixedOffset>> {
    let parsed = DateTime::parse_from_str(data, "%Y-%m-%d %H:%M:%S %z")?;
    Ok(parsed)
}

named!(log_lines(&str) -> Vec<Line>,
    many0!(
        terminated!(log_line, opt!(line_ending))
    )
);

named!(log_line(&str) -> Line,
        alt!(log_message | log_joined_channel | log_system_message | log_begin | log_end | log_separator | log_other)
);

named!(log_separator(&str) -> Line,
    map!(
        tag!("-"),
        |_| Line::Separator
    )
);

named!(log_other(&str) -> Line,
    map!(
        not_line_ending,
        |text| Line::Other(text.to_string())
    )
);

named!(log_begin(&str) -> Line,
    map!(
        preceded!(tag!("# Log started: "), not_line_ending),
        |time| Line::BeginLog(parse_date_time(time).unwrap())
    )
);

named!(log_end(&str) -> Line,
    map!(
        preceded!(tag!("# Log closed: "), not_line_ending),
        |time| Line::EndLog(parse_date_time(time).unwrap())
    )
);

named!(log_message(&str) -> Line,
    map!(
        do_parse!(
            time: message_timestamp >>
            space >>
            sender: message_sender >>
            space >>
            message: not_line_ending >>
            (time, sender, message)
        ),
        |tuple| Line::Message { time: tuple.0, sender: tuple.1, message: tuple.2.to_string() }
    )
);

named!(log_joined_channel(&str) -> Line,
    map!(
        do_parse!(
            time: message_timestamp >>
            space >>
            tag!("You have joined") >>
            space >>
            channel: not_line_ending >>
            (time, channel)
        ),
        |tuple| Line::JoinedChannel { time: tuple.0, channel: tuple.1.to_string() }
    )
);

named!(log_system_message(&str) -> Line,
    map!(
        do_parse!(
            time: message_timestamp >>
            space >>
            message: not_line_ending >>
            (time, message)
        ),
        |tuple| Line::SystemMessage { time: tuple.0, message: tuple.1.to_string() }
    )
);

named!(message_sender(&str) -> MessageSender,
    map!(
        do_parse!(
            tag!("<") >>
            modifiers: many0!(alt!(
                map!(char!('+'), |_| MessageFlag::Prime) |
                map!(char!('@'), |_| MessageFlag::Moderator) |
                map!(char!('%'), |_| MessageFlag::Subscriber) |
                map!(char!('~'), |_| MessageFlag::Broadcaster)
            )) >>
            name: is_not_s!(">") >>
            tag!(">") >>
            (name, modifiers)
        ),
        |tuple| MessageSender { name: tuple.0.to_owned(), modifiers: tuple.1 }
    )
);

named!(message_timestamp(&str) -> MessageTimestamp,
    map!(
        do_parse!(
            tag!("[") >>
            date: opt!(
                map!(do_parse!(
                        year: digit >>
                        tag!("-") >>
                        month: digit >>
                        tag!("-") >>
                        day: digit >>
                        (year, month, day)
                    ),
                    |ymd| NaiveDate::from_ymd(i32::from_str(ymd.0).unwrap(),
                                              u32::from_str(ymd.1).unwrap(),
                                              u32::from_str(ymd.2).unwrap())
                )
            ) >>
            opt!(tag!(" ")) >>
            time: map!(
                do_parse!(
                    hour: digit >>
                    tag!(":") >>
                    minute: digit >>
                    tag!(":") >>
                    second: digit >>
                    (hour, minute, second)
                ),
                |hms| NaiveTime::from_hms(u32::from_str(hms.0).unwrap(),
                                          u32::from_str(hms.1).unwrap(),
                                          u32::from_str(hms.2).unwrap())
            ) >>
            tag!("]") >>
            (date, time)
        ),
        |dt| MessageTimestamp { date: dt.0, time: dt.1 }
    )
);

impl LogParser for ChattyParser {
    fn parse<T: BufRead + ? Sized>(collector: &mut Collector, input: T) -> Result<()> where T: Sized {
        let mut log_time: Option<DateTime<FixedOffset>> = None;
        let mut channel: Option<String> = None;

        for (line_num, line) in input.lines().enumerate() {
            let line = line?;
            // Parse line
            let parse_result: IResult<&str, Line> = log_line(line.as_ref());

            // Handle results
            match parse_result {
                IResult::Done(_, line) => {
                    match line {
                        Line::BeginLog(time) | Line::EndLog(time) => {
                            log_time = Some(time);
                        },
                        Line::Message { time, message, sender } => {
                            let channel = channel.as_ref().ok_or(ErrorKind::MissingJoinChannel)?;
                            let prev_time = log_time.ok_or(ErrorKind::MissingBeginTimestamp)?;
                            log_time = Some(time.into_datetime(prev_time)?);
                            collector.add_message(RawMessage {
                                message,
                                channel: channel.to_owned(),
                                nick: sender.name,
                                sent_at: log_time.unwrap().naive_utc(),
                                flags: sender.modifiers
                            })?;
                        },
                        Line::SystemMessage { time, .. } => {
                            let prev_time = log_time.ok_or(ErrorKind::MissingBeginTimestamp)?;
                            log_time = Some(time.into_datetime(prev_time)?);
                        },
                        Line::JoinedChannel { channel: joined, time } => {
                            let prev_time = log_time.ok_or(ErrorKind::MissingBeginTimestamp)?;
                            log_time = Some(time.into_datetime(prev_time)?);
                            channel = Some(joined);
                        },
                        Line::Separator => {},
                        Line::Other(msg) => {
                            println!("Unknown message type encountered, ignoring line {}", line_num + 1);
                            println!("{}", msg);
                        }
                    }
                }
                IResult::Incomplete(_) => Err(ErrorKind::IncompleteLineError(line_num + 1))?,
                IResult::Error(err) => Err(ErrorKind::ParseError(line_num + 1, err))?,
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use collector::VecCollector;
    use std::io::BufReader;

    #[test]
    fn parse_full() {
        let mut collector = VecCollector::new();
        let text: &str = indoc!("
        # Log started: 2017-10-05 23:40:00 +0200
        [23:45:00] You have joined #_cerebot
        [00:10:00] <@+JohnDoe> test message 1
        [01:10:00] <@+JohnDoe> test message 2
        [02:10:00] <@+JohnDoe> test message 3
        # Log closed: 2017-10-08 15:19:46 +0200
        ");
        ChattyParser::parse(&mut collector, BufReader::new(text.as_bytes()))
            .unwrap();

        println!("{:?}", collector.messages)
    }

    #[test]
    fn parse_log_begin() {
        let time = Date::from_utc(
            NaiveDate::from_ymd(2016, 4, 11),
            FixedOffset::east(2*3600)
        ).and_hms(13, 1, 58);
        let text = "# Log started: 2016-04-11 13:01:58 +0200";
        assert_eq!(log_begin(text).unwrap().1, Line::BeginLog(time))
    }

    #[test]
    fn parse_log_close() {
        let time = Date::from_utc(
            NaiveDate::from_ymd(2016, 4, 11),
            FixedOffset::east(2*3600)
        ).and_hms(13, 1, 58);
        let text = "# Log closed: 2016-04-11 13:01:58 +0200";
        assert_eq!(log_end(text).unwrap().1, Line::EndLog(time))
    }

    #[test]
    fn parse_sender() {
        let regular = "<JohnDoe>";
        assert_eq!(message_sender(regular).unwrap(),
                   ("", MessageSender {
                       name: "JohnDoe".to_owned(), modifiers: vec!()
                   })
        );
    }

    #[test]
    fn parse_sender_modifiers() {
        let prime = "<+JohnDoe>";
        let moderator = "<@JohnDoe>";
        let broadcaster = "<~JohnDoe>";
        let multi = "<~+@%JohnDoe>";

        assert_eq!(message_sender(moderator).unwrap(),
                   ("", MessageSender { name: "JohnDoe".to_owned(), modifiers: vec!(MessageFlag::Moderator) })
        );

        assert_eq!(message_sender(prime).unwrap(),
                   ("", MessageSender { name: "JohnDoe".to_owned(), modifiers: vec!(MessageFlag::Prime) })
        );

        assert_eq!(message_sender(broadcaster).unwrap(),
                   ("", MessageSender { name: "JohnDoe".to_owned(), modifiers: vec!(MessageFlag::Broadcaster) })
        );

        assert_eq!(message_sender(multi).unwrap(),
                   ("", MessageSender {
                       name: "JohnDoe".to_owned(), modifiers: vec!(
                           MessageFlag::Broadcaster,
                           MessageFlag::Prime,
                           MessageFlag::Moderator,
                           MessageFlag::Subscriber,
                       )
                   })
        );
    }

    #[test]
    fn parse_message_timestamp() {
        let text_time_only = "[22:05:44]";
        let time = NaiveTime::from_hms(22, 5, 44);
        let text_date_time = "[2017-10-08 22:05:44]";
        let date = NaiveDate::from_ymd(2017, 10, 8);

        assert_eq!(message_timestamp(text_time_only).unwrap(), ("", MessageTimestamp {
            time,
            date: None,
        }));

        assert_eq!(message_timestamp(text_date_time).unwrap(), ("", MessageTimestamp {
            time,
            date: Some(date),
        }));
    }

    #[test]
    fn parse_log_message() {
        let expected = Line::Message {
            time: MessageTimestamp {
                time: NaiveTime::from_hms(22, 5, 44),
                date: Some(NaiveDate::from_ymd(2017, 10, 8)),
            },
            message: "this is a test".to_owned(),
            sender: MessageSender {
                name: "JohnDoe".to_owned(), modifiers: vec!(MessageFlag::Prime)
            },
        };

        let message = "[2017-10-08 22:05:44] <+JohnDoe> this is a test";

        assert_eq!(log_message(message).unwrap(), ("", expected))
    }

    #[test]
    fn parse_lines() {
        let lines = indoc!("
            # Log started: 2017-10-08 22:05:40 +0200
            [2017-10-08 22:05:40] Joining #some_channel..
            [2017-10-08 22:05:44] <+JohnDoe> this is a test
        ");

        let parsed = log_lines(lines).unwrap();
        assert_eq!(parsed.1[2], Line::Message {
            time: MessageTimestamp {
                time: NaiveTime::from_hms(22, 5, 44),
                date: Some(NaiveDate::from_ymd(2017, 10, 8)),
            },
            message: "this is a test".to_owned(),
            sender: MessageSender {
                name: "JohnDoe".to_owned(), modifiers: vec!(MessageFlag::Prime)
            },
        })
    }
}
