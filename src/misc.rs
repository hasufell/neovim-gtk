use std::borrow::Cow;
use std::mem;

use lazy_static::lazy_static;

use percent_encoding::percent_decode;
use regex::Regex;

use crate::shell;

/// Split comma separated parameters with ',' except escaped '\\,'
pub fn split_at_comma(source: &str) -> Vec<String> {
    let mut items = Vec::new();

    let mut escaped = false;
    let mut item = String::new();

    for ch in source.chars() {
        if ch == ',' && !escaped {
            item = item.replace("\\,", ",");

            let mut new_item = String::new();
            mem::swap(&mut item, &mut new_item);

            items.push(new_item);
        } else {
            item.push(ch);
        }
        escaped = ch == '\\';
    }

    if !item.is_empty() {
        items.push(item.replace("\\,", ","));
    }

    items
}

/// Escape special ASCII characters with a backslash.
pub fn escape_filename<'t>(filename: &'t str) -> Cow<'t, str> {
    lazy_static! {
        static ref SPECIAL_CHARS: Regex = if cfg!(target_os = "windows") {
            // On Windows, don't escape `:` and `\`, as these are valid components of the path.
            Regex::new(r"[[:ascii:]&&[^0-9a-zA-Z._:\\-]]").unwrap()
        } else {
            // Similarly, don't escape `/` on other platforms.
            Regex::new(r"[[:ascii:]&&[^0-9a-zA-Z._/-]]").unwrap()
        };
    }
    SPECIAL_CHARS.replace_all(&*filename, r"\$0")
}

/// Decode a file URI.
///
///   - On UNIX: `file:///path/to/a%20file.ext` -> `/path/to/a file.ext`
///   - On Windows: `file:///C:/path/to/a%20file.ext` -> `C:\path\to\a file.ext`
pub fn decode_uri(uri: &str) -> Option<String> {
    let path = match uri.split_at(8) {
        ("file:///", path) => path,
        _ => return None,
    };
    let path = percent_decode(path.as_bytes()).decode_utf8().ok()?;
    if cfg!(target_os = "windows") {
        lazy_static! {
            static ref SLASH: Regex = Regex::new(r"/").unwrap();
        }
        Some(String::from(SLASH.replace_all(&*path, r"\")))
    } else {
        Some("/".to_owned() + &path)
    }
}

/// info text
pub fn about_comments() -> String {
    format!(
        "Build on top of neovim\n\
         Minimum supported neovim version: {}",
        shell::MINIMUM_SUPPORTED_NVIM_VERSION
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_comma_split() {
        let res = split_at_comma("a,b");
        assert_eq!(2, res.len());
        assert_eq!("a", res[0]);
        assert_eq!("b", res[1]);

        let res = split_at_comma("a,b\\,c");
        assert_eq!(2, res.len());
        assert_eq!("a", res[0]);
        assert_eq!("b,c", res[1]);
    }
}


#[macro_export]
/// Try to unwrap a `Result<T, E>`. If there is a value `T`, yield it,
/// otherwise print a warning and return from the function with the given value.
macro_rules! try_wr {
    ($expr:expr, $ret:expr) => (match $expr {
        ::std::result::Result::Ok(val) => val,
        ::std::result::Result::Err(err) => {
            warn!("{:?}", err);
            return $ret;
        },
    });
    ($expr:expr, $ret:expr, $fmt:expr) => (match $expr {
        ::std::result::Result::Ok(val) => val,
        ::std::result::Result::Err(err) => {
            warn!("Original error: {:?}", err);
            warn!($fmt);
            return $ret;
        },
    });
    ($expr:expr, $ret:expr, $fmt:expr, $($arg:tt)+) => (match $expr {
        ::std::result::Result::Ok(val) => val,
        ::std::result::Result::Err(err) => {
            warn!("Original error: {:?}", err);
            warn!(format!($fmt, $(arg)+));
            return $ret;
        },
    })
}

#[macro_export]
/// Try to unwrap a `Result<T, E>`. If there is a value `T`, yield it,
/// otherwise print a warning and `return ()` from the function.
macro_rules! try_w {
    ($expr:expr) => {
        try_wr!($expr, ())
    };
    ($expr:expr, $fmt:expr, $($arg:tt)+) => {
        try_wr!($expr, (), $fmt, $(arg)+)
    };
    ($expr:expr, $fmt:expr) => {
        try_wr!($expr, (), $fmt)
    }
}

#[macro_export]
/// Try to unwrap a `Result<T, E>`. If there is a value `T`, yield it,
/// otherwise return from the function with the given value.
macro_rules! try_r {
    ($expr:expr, $ret:expr) => (match $expr {
        ::std::result::Result::Ok(val) => val,
        ::std::result::Result::Err(_) => {
            return $ret;
        },
    });
}


#[macro_export]
/// Try to unwrap a `Result<T, E>`. If there is a value `T`, yield it,
/// otherwise print an error and exit the program.
macro_rules! try_e {
    ($expr:expr) => (match $expr {
        ::std::result::Result::Ok(val) => val,
        ::std::result::Result::Err(err) => {
            error!("{:?}", err);
            ::std::process::exit(1);
        },
    });
    ($expr:expr, $fmt:expr) => (match $expr {
        ::std::result::Result::Ok(val) => val,
        ::std::result::Result::Err(err) => {
            error!("Original error: {:?}", err);
            error!($fmt);
            std::process::exit(1);
        },
    });
    ($expr:expr, $fmt:expr, $($arg:tt)+) => (match $expr {
        ::std::result::Result::Ok(val) => val,
        ::std::result::Result::Err(err) => {
            error!("Original error: {:?}", err);
            error!(format!($fmt, $(arg)+));
            std::process::exit(1);
        },
    })
}

#[macro_export]
/// Unwraps a `Result<T, E>` by yielding a value of the same type
/// for either case.
macro_rules! unwrap_any {
    ($expr:expr, $fmt_ok:expr, $fmt_err:expr) => (match $expr {
        ::std::result::Result::Ok(val) => $fmt_ok,
        ::std::result::Result::Err(err) => $fmt_err,
    })

}

#[macro_export]
/// Checks the given expression evaluates to true, otherwise
/// prints warning and returns with the given value.
macro_rules! check {
    ($expr:expr, $parent:expr, $ret:expr) => (match $expr {
        true => (),
        false => {
            warn!("{:?}", $parent);
            return $ret;
        },
    });
}
