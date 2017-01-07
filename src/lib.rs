//! A cache manager for the publicsuffix crate

extern crate publicsuffix;
#[macro_use]
extern crate lazy_static;
extern crate parking_lot;
#[macro_use]
extern crate slog;
#[macro_use]
extern crate slog_scope;

#[cfg(test)]
mod tests;

use std::thread;
use std::time::Duration;

use publicsuffix::errors::*;
use publicsuffix::{List, IntoUrl};
use parking_lot::{RwLock, RwLockReadGuard};
use slog::Logger;
use slog_scope::set_global_logger;

lazy_static! {
    static ref LIST: RwLock<List> = RwLock::new(List::empty());
}

/// The lock guard for the list
///
/// It derefences into an instance of `publicsuffix::List`.
pub type ListGuard<'a> = RwLockReadGuard<'a, List>;

fn update_list(url: &str) -> Result<()> {
    info!("updating the public suffix list from {}", url);
    let mut list = LIST.write();
    *list = List::from_url(url)?;
    info!("the list has been updated successfully");
    Ok(())
}

/// Initialise the list
///
/// Call from your `main` or `run` function. It fetches the list from `url`
/// using a certain interval.
///
/// ## Example
///
/// ```rust,norun
/// extern crate psl;
/// extern crate publicsuffix;
///
/// use publicsuffix::LIST_URL;
/// use std::time::Duration;
///
/// fn main() {
///     // Update the list every week
///     psl::init(LIST_URL, None).unwrap();
///
///     // Or update every 2 weeks
///     psl::init(LIST_URL, Duration::from_secs(60 * 60 * 24 * 7 * 2)).unwrap();
/// }
/// ```
///
/// If it fails to fetch the list for the first time it will return an error.
/// After it successfully fetches the list for the first time it will try to download
/// an update at `every` interval retrying every 5 minutes if it fails.
///
/// If you are using this in a long running server, I highly recommend you set up a logger
/// using `set_logger` so you will know if updates start failing at some point in future.
pub fn init<U, D>(url: U, every: D) -> Result<()>
    where U: IntoUrl,
          D: Into<Option<Duration>>
{
    let url = url.into_url()?.into_string();
    update_list(&url)?;
    // default to updating the list every week
    let freq = every.into().unwrap_or(Duration::from_secs(60 * 60 * 24 * 7));
    thread::spawn(move || {
        loop {
            thread::sleep(freq);
            loop {
                match update_list(&url) {
                    Ok(_) => break,
                    Err(error) => {
                        warn!("failed to update the list: {}", error);
                        info!("will try again in 5 minutes");
                        thread::sleep(Duration::from_secs(300));
                    }
                }
            }
        }
    });
    Ok(())
}

/// Gets an instance of the list from the cache
///
/// ListGuard derefs into `publicsuffix::List` so you can call the
/// list methods directly.
///
/// ## Example
/// 
/// ```rust,norun
/// # extern crate psl;
/// # extern crate publicsuffix;
/// # fn foo() -> Result<(), ::publicsuffix::errors::Error> {
/// let domain = psl::get().parse_domain("example.com")?;
/// # Ok(())
/// # }
/// # fn main() {}
/// ```
pub fn get<'a>() -> ListGuard<'a> {
    LIST.read()
}

/// Setup an `slog` logger
pub fn set_logger(l: &Logger)
{
    set_global_logger(l.clone());
}
