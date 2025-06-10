use ansic::ansi;

pub const RADOS: &'static str = ansi!(red bold);
pub const USER: &'static str = ansi!(yellow bold);
pub const RESET: &'static str = ansi!(reset);

pub const RBRRED: &'static str = ansi!(reset br.red);
pub const BBRRED: &'static str = ansi!(bold br.red);
pub const BRED: &'static str = ansi!(bold red);
