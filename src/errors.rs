use nom::ErrorKind as NomErrorKind;

error_chain! {
    types {
        Error, ErrorKind, ResultExt, Result;
    }

    foreign_links {
        Diesel(::diesel::result::Error);
        IoError(::std::io::Error);
        ChronoParseError(::chrono::ParseError);
        NomError(::nom::ErrorKind);
    }

    errors {
        TimestampError {
            description("Error determining full message timestamp")
        }
        MissingBeginTimestamp {
            description("Log contained no beginning timestamp")
        }
        MissingJoinChannel {
            description("No 'Joined channel' line found before first message")
        }
        ParseError(line_num: usize, cause: NomErrorKind) {
            description("Parsing Error")
            display("Parsing Error: '{}' in line {}", cause, line_num)
        }
        IncompleteLineError(line_num: usize) {
            description("Line ended prematurely, parsing incomplete")
            display("Parsing Error: Line {} was incomplete", line_num)
        }
    }
}
