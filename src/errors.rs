use diesel::result::Error as DieselError;

error_chain! {
    types {
        Error, ErrorKind, ResultExt, Result;
    }

    foreign_links {
        Diesel(DieselError);
    }

    errors {}
}
