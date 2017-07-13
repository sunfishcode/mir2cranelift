#![feature(rustc_private, link_args)]

extern crate env_logger;
extern crate getopts;
#[macro_use]
extern crate log;
extern crate mir2cretonne;
extern crate rustc;
extern crate rustc_driver;

use getopts::Options;
use mir2cretonne::trans::{self, Mir2CretonneTransOptions};
use rustc::session::{Session, CompileIncomplete};
use rustc_driver::{driver, CompilerCalls};

struct Mir2CretonneCompilerCalls {
    options: Mir2CretonneTransOptions,
}

impl Mir2CretonneCompilerCalls {
    fn new(options: Mir2CretonneTransOptions) -> Mir2CretonneCompilerCalls {
        Mir2CretonneCompilerCalls { options: options }
    }
}

impl<'a> CompilerCalls<'a> for Mir2CretonneCompilerCalls {
    fn build_controller(
        &mut self,
        _: &Session,
        _: &getopts::Matches,
    ) -> driver::CompileController<'a> {
        let mut control = driver::CompileController::basic();

        let options = self.options.clone();

        control.after_analysis.stop = rustc_driver::Compilation::Stop;
        control.after_analysis.callback = Box::new(move |state| {
            state.session.abort_if_errors();

            let entry_fn = state.session.entry_fn.borrow();
            let entry_fn = if let Some((node_id, _)) = *entry_fn {
                Some(node_id)
            } else {
                None
            };
            trans::trans_crate(
                state.tcx.expect("type context needed").global_tcx(),
                entry_fn,
                &options,
            ).expect("error translating crate")
        });

        control
    }
}

enum ReifiedOpt<'a> {
    OptFlag {
        short_name: &'a str,
        long_name: &'a str,
        desc: &'a str,
    },
    OptOpt {
        short_name: &'a str,
        long_name: &'a str,
        desc: &'a str,
        hint: &'a str,
    },
}

fn short_name<'a>(opt: &'a ReifiedOpt<'a>) -> &'a str {
    match *opt {
        ReifiedOpt::OptFlag { short_name, .. } |
        ReifiedOpt::OptOpt { short_name, .. } => short_name,
    }
}

fn long_name<'a>(opt: &'a ReifiedOpt<'a>) -> &'a str {
    match *opt {
        ReifiedOpt::OptFlag {
            short_name: _,
            long_name,
            ..
        } |
        ReifiedOpt::OptOpt {
            short_name: _,
            long_name,
            ..
        } => long_name,
    }
}

fn main() {
    env_logger::init().unwrap();

    let mut reified = Vec::<ReifiedOpt>::new();
    reified.push(ReifiedOpt::OptOpt {
        short_name: "o",
        long_name: "",
        desc: "write output to FILE",
        hint: "FILE",
    });
    reified.push(ReifiedOpt::OptFlag {
        short_name: "q",
        long_name: "",
        desc: "do not print the compiled wast module",
    });
    reified.push(ReifiedOpt::OptFlag {
        short_name: "h",
        long_name: "help",
        desc: "display this help message",
    });

    let mut opts = Options::new();
    for reified_opt in &reified {
        match *reified_opt {
            ReifiedOpt::OptFlag {
                short_name,
                long_name,
                desc,
            } => {
                opts.optflag(short_name, long_name, desc);
            }
            ReifiedOpt::OptOpt {
                short_name,
                long_name,
                desc,
                hint,
            } => {
                opts.optopt(short_name, long_name, desc, hint);
            }
        }
    }

    let mut rustc_args = Vec::new();
    let mut mir2cretonne_args = Vec::new();

    fn find_mir2cretonne_arg<'a>(
        s: &str,
        opts: &'a [ReifiedOpt<'a>],
    ) -> Option<&'a ReifiedOpt<'a>> {
        for o in opts {
            if s.starts_with("--") && &s[2..] == long_name(o) {
                return Some(o);
            }
            if s.starts_with('-') && &s[1..] == short_name(o) {
                return Some(o);
            }
        }
        None
    };

    let args: Vec<String> = std::env::args().collect();
    info!("command line: {:?}", args);

    let mut argv = std::env::args().peekable();
    while let Some(arg) = argv.next() {
        match find_mir2cretonne_arg(&arg, &reified) {
            Some(opt) => {
                mir2cretonne_args.push(arg);

                match *opt {
                    ReifiedOpt::OptOpt { .. } => {
                        mir2cretonne_args.push(argv.next().expect("missing required argument"))
                    }
                    ReifiedOpt::OptFlag { .. } => (),
                }
            }
            None => rustc_args.push(arg),
        }
    }
    info!("mir2cretonne args: {:?}", mir2cretonne_args);
    info!("rustc args: {:?}", rustc_args);

    let mut options = Mir2CretonneTransOptions::new();

    let matches = opts.parse(&mir2cretonne_args[..]).expect(
        "could not parse command line arguments",
    );

    if matches.opt_present("h") {
        let brief = format!("Usage: {} [options]", args[0]);
        print!("{}", opts.usage(&brief));
        return;
    }
    if matches.opt_present("o") {
        options.binary_output_path = matches.opt_str("o");
    }
    if matches.opt_present("q") {
        options.print = false;
    }

    let mut compiler_calls = Mir2CretonneCompilerCalls::new(options);
    let (result, _) = rustc_driver::run_compiler(&rustc_args, &mut compiler_calls, None, None);
    match result {
        Err(CompileIncomplete::Stopped) => (),
        Ok(n) => {
            panic!("Unexpected success {:?}", n);
        }
        Err(n) => {
            panic!("Error {:?}", n);
        }
    }
}
