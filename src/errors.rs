use diesel::result::Error as DieselError;
use std::io::Error as IOError;
use chrono::ParseError as ChronoParseError;

error_chain! {
    types {
        Error, ErrorKind, ResultExt, Result;
    }

    foreign_links {
        Diesel(DieselError);
        IO(IOError);
        ChronoParseError(ChronoParseError);
    }

    errors {}
}
