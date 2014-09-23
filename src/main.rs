#![crate_name = "clog"]
#![comment = "A conventional changelog generator"]
#![license = "MIT"]
#![feature(macro_rules, phase)]

extern crate regex;
#[phase(plugin)]
extern crate regex_macros;
extern crate serialize;
#[phase(plugin)] extern crate docopt_macros;
extern crate docopt;
extern crate time;

use git::{ LogReaderConfig, get_commits, get_latest_tag };
use log_writer::{ LogWriter, LogWriterOptions };
use section_builder::build_sections;
use std::io::{File, Open, Write};
use docopt::FlagParser;
use time::get_time;

mod common;
mod git;
mod log_writer;
mod section_builder;

docopt!(Args, "clog

Usage:
  clog [--repository=<link> --setversion=<version> --subtitle=<subtitle> 
        --from=<from> --to=<to> --from-latest-tag]

Options:
  -h --help               Show this screen.
  --version               Show version
  -r --repository=<link>  e.g https://github.com/thoughtram/clog
  --setversion=<version>  e.g. 0.1.0
  --subtitle=<subtitle>   e.g. crazy-release-name
  --from=<from>           e.g. 12a8546
  --to=<to>               e.g. 8057684
  --from-latest-tag       uses the latest tag as starting point. Ignores other --from parameter")

fn main () {

    let start_nsec = get_time().nsec;
    let args: Args = FlagParser::parse().unwrap_or_else(|e| e.exit());

    let log_reader_config = LogReaderConfig {
        grep: "^feat|^fix|BREAKING'".to_string(),
        format: "%H%n%s%n%b%n==END==".to_string(),
        from: if args.flag_from_latest_tag { get_latest_tag() } else { args.flag_from },
        to: args.flag_to
    };

    let commits = get_commits(log_reader_config);

    let sections = build_sections(commits.clone());
    let contents = File::open(&Path::new("changelog.md")).read_to_string().unwrap();

    let mut file = File::open_mode(&Path::new("changelog.md"), Open, Write).ok().unwrap();
    let mut writer = LogWriter::new(&mut file, LogWriterOptions { 
        repository_link: args.flag_repository,
        version: args.flag_setversion,
        subtitle: args.flag_subtitle
    });

    writer.write_header();
    writer.write_section("Bug Fixes", &sections.fixes);
    writer.write_section("Features", &sections.features);
    writer.write(contents.as_slice());

    let end_nsec = get_time().nsec;
    let elapsed_mssec = (end_nsec - start_nsec) / 1000000;
    println!("changelog updated. (took {} ms)", elapsed_mssec);
}
