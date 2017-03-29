// utility functions
// this module must not import anything from the project

macro_rules! log {
    ($log_level:ident, $fmt:expr) => ({
        if $log_level.debug {
            println!($fmt);
        }
    });
    ($log_level:ident, $fmt:expr, $($arg:tt)*) => ({
        if $log_level.debug {
            println!($fmt, $($arg)*);
        }
    })
}
