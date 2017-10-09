#![allow(dead_code)]

use std::io::BufRead;
use chrono::prelude::*;
use collector::{Collector};
use nom::{IResult, not_line_ending, digit, space, line_ending};
use errors::Result;
use std::str::FromStr;

pub trait LogParser {
    fn parse<T: BufRead + ? Sized>(collector: &mut Collector, input: T) -> Result<()> where T: Sized;
}

pub struct ChattyParser {}

#[derive(Debug, Clone, PartialEq)]
struct MessageTimestamp {
    pub time: NaiveTime,
    pub date: Option<NaiveDate>
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
    Other,
}

#[derive(Debug, Clone, PartialEq)]
struct MessageSender {
    pub name: String,
    pub modifiers: Vec<MessageModifier>
}

#[derive(Debug, Clone, PartialEq)]
enum MessageModifier {
    Broadcaster,
    Moderator,
    Prime,
    Subscriber
}

fn parse_date_time(data: &str) -> Result<DateTime<FixedOffset>> {
    let parsed = DateTime::parse_from_str(data, "%Y-%m-%d %H:%M:%S %z")?;
    Ok(parsed)
}

named!(log_lines(&str) -> Vec<Line>,
    many0!(
        terminated!(alt!(log_message | log_system_message | log_begin | log_end), opt!(line_ending))
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
                map!(char!('+'), |_| MessageModifier::Prime) |
                map!(char!('@'), |_| MessageModifier::Moderator) |
                map!(char!('%'), |_| MessageModifier::Subscriber) |
                map!(char!('~'), |_| MessageModifier::Broadcaster)
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
        let mut log_begin_time: Option<DateTime<Utc>> = None;
        for line in input.lines() {
            match log_begin(line.unwrap().as_ref()) {
                IResult::Done(i, o) => {
                    println!("{:?}", i);
                    println!("{:?}", o);
                }
                err => println!("{:?}", err)
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

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
        let regular = "<joehndoe>";
        assert_eq!(message_sender(regular).unwrap(),
                   ("", MessageSender {
                       name: "joehndoe".to_owned(), modifiers: vec!()
                   })
        );
    }

    #[test]
    fn parse_sender_modifiers() {
        let prime = "<+joehndoe>";
        let moderator = "<@joehndoe>";
        let broadcaster = "<~joehndoe>";
        let multi = "<~+@%joehndoe>";

        assert_eq!(message_sender(moderator).unwrap(),
                   ("", MessageSender { name: "joehndoe".to_owned(), modifiers: vec!(MessageModifier::Moderator) })
        );

        assert_eq!(message_sender(prime).unwrap(),
                   ("", MessageSender { name: "joehndoe".to_owned(), modifiers: vec!(MessageModifier::Prime) })
        );

        assert_eq!(message_sender(broadcaster).unwrap(),
                   ("", MessageSender { name: "joehndoe".to_owned(), modifiers: vec!(MessageModifier::Broadcaster) })
        );

        assert_eq!(message_sender(multi).unwrap(),
                   ("", MessageSender {
                       name: "joehndoe".to_owned(), modifiers: vec!(
                           MessageModifier::Broadcaster,
                           MessageModifier::Prime,
                           MessageModifier::Moderator,
                           MessageModifier::Subscriber,
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
                name: "JohnDoe".to_owned(), modifiers: vec!(MessageModifier::Prime)
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
                name: "JohnDoe".to_owned(), modifiers: vec!(MessageModifier::Prime)
            },
        })
    }
}
