//! A cache manager for the publicsuffix crate

extern crate publicsuffix;
#[macro_use]
extern crate lazy_static;
extern crate parking_lot;
#[macro_use]
extern crate slog;
extern crate app_dirs;

#[cfg(test)]
mod tests;

use std::thread;
use std::io::Write;
use std::path::PathBuf;
use std::time::Duration;
use std::fs::{self, File};

use publicsuffix::errors::*;
use publicsuffix::{List, IntoUrl};
use parking_lot::{RwLock, RwLockReadGuard};
use slog::Logger;
use app_dirs::{AppDataType, AppInfo, app_root};

lazy_static! {
    static ref LIST: RwLock<List> = RwLock::new(List::empty());
}

const APP_INFO: AppInfo = AppInfo {
    name: "publicsuffix",
    author: "mozilla",
};

/// The lock guard for the list
///
/// It derefences into an instance of `publicsuffix::List`.
pub type ListGuard<'a> = RwLockReadGuard<'a, List>;

#[derive(Debug, Clone)]
struct Cache {
    url: String,
    freq: Duration,
    logger : Logger,
}

impl Cache {
    fn new(url: String, freq: Duration, logger : Logger) -> Cache {
        Cache {
            url: url,
            freq: freq,
            logger: logger,
        }
    }

    fn path(&self) -> Result<PathBuf> {
        app_root(AppDataType::UserCache, &APP_INFO)
            .chain_err(|| "error accessing the data directory")
            .and_then(|mut file| {
                file.push("list.dat");
                Ok(file)
            })
    }

    fn update(&self) -> Result<()> {
        let mut list = LIST.write();
        *list = self.list()?;
        info!(self.logger, "the list has been updated successfully");
        Ok(())
    }

    fn list(&self) -> Result<List> {
        match self.path() {
            Ok(path) => {
                if path.is_file() {
                    let last_update = path.metadata()?.modified()?;
                    let elapsed = last_update.elapsed()
                        .chain_err(|| "failed to get elapsed time")?;
                    if elapsed > self.freq {
                        self.download_and_save()
                            .or_else(|error| {
                                info!(self.logger, "failed to download file: {}", error);
                                info!(self.logger, "updating the public suffix list from {}", path.to_str().unwrap());
                                List::from_path(path)
                            })
                    } else {
                        info!(self.logger, "updating the public suffix list from {}", path.to_str().unwrap());
                        List::from_path(path)
                            .or_else(|error| {
                                info!(self.logger, "failed to retrieve the list from local cache: {}", error);
                                info!(self.logger, "updating the public suffix list from {}", self.url);
                                self.download_and_save()
                            })
                    }
                } else {
                    self.download_and_save()
                }
            }
            Err(error) => {
                warn!(self.logger, "failed querying cache path: {}", error);
                self.download_and_save()
            }
        }
    }

    fn download_and_save(&self) -> Result<List> {
        info!(self.logger, "updating the public suffix list from {}", self.url);
        let list = List::from_url(&self.url)?;
        if let Err(error) = self.save(&list) {
            warn!(self.logger, "failed to save the list to disk: {}", error);
        }
        Ok(list)
    }

    fn save(&self, list: &List) -> Result<()> {
        let file = self.path()?;
        if list.all().is_empty() {
            fs::remove_file(file)?;
            return Ok(());
        }
        let mut data = String::with_capacity(list.all().len());
        if !list.icann().is_empty() {
            data.push_str("// ===BEGIN ICANN DOMAINS===\n");
            for rule in list.icann() {
                data.push_str(&format!("{}\n", rule));
            }
        }
        if !list.private().is_empty() {
            data.push_str("// ===BEGIN PRIVATE DOMAINS===\n");
            for rule in list.private() {
                data.push_str(&format!("{}\n", rule));
            }
        }
        let mut file = File::create(file)?;
        file.write_all(data.as_bytes())?;
        file.sync_all()?;
        Ok(())
    }
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
///     psl::init(LIST_URL, None, None).unwrap();
///
///     // Or update every 2 weeks
///     psl::init(LIST_URL, Duration::from_secs(60 * 60 * 24 * 7 * 2), None).unwrap();
/// }
/// ```
///
/// If it fails to fetch the list for the first time it will return an error.
/// After it successfully fetches the list for the first time it will try to download
/// an update at `every` interval retrying every 5 minutes if it fails.
///
/// If you are using this in a long running server, I highly recommend you set up a `logger`
/// so you will know if updates start failing at some point in future.
pub fn init<U, D, L>(url: U, every: D, logger : L) -> Result<()>
    where U: IntoUrl,
          D: Into<Option<Duration>>,
          L: Into<Option<Logger>>
{
    let logger = logger.into().unwrap_or(slog::Logger::root(slog::Discard, o!()));

    let url = url.into_url()?.into_string();
    // default to updating the list every week
    let freq = every.into().unwrap_or(Duration::from_secs(60 * 60 * 24 * 7));
    let cache = Cache::new(url, freq, logger.clone());
    cache.update()?;
    thread::spawn(move || {
        loop {
            thread::sleep(cache.freq);
            loop {
                match cache.update() {
                    Ok(_) => break,
                    Err(error) => {
                        warn!(logger, "failed to update the list: {}", error);
                        info!(logger, "will try again in 5 minutes");
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
