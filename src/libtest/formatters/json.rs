// Copyright 2012-2017 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::io;
use serde_json::Value as Json;

use ::*;

pub(crate) struct JsonFormatter<T> {
    out: OutputLocation<T>,
}

impl<T: Write> JsonFormatter<T> {
    pub fn new(out: OutputLocation<T>) -> Self {
        Self { out }
    }

    fn write_message(&mut self, s: &Json) -> io::Result<()> {
        serde_json::to_writer(&mut self.out, s).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        self.out.write_all(b"\n")
    }
}

impl<T: Write> OutputFormatter for JsonFormatter<T> {
    fn write_run_start(&mut self, test_count: usize) -> io::Result<()> {
        self.write_message(&json!({ "type": "suite", "event": "started", "test_count": test_count, }))
    }

    fn write_test_start(&mut self, desc: &TestDesc) -> io::Result<()> {
        self.write_message(&json!({ "type": "test", "event": "started", "name": desc.name.as_str(), }))
    }

    fn write_result(
        &mut self,
        desc: &TestDesc,
        result: &TestResult,
        stdout: &[u8],
    ) -> io::Result<()> {
        match *result {
            TrOk => self.write_message(&json!({
                "type": "test",
                "name": desc.name.as_str(),
                "event": "ok",
            })),
            TrFailed => self.write_message(&json!({
                "type": "test",
                "name": desc.name.as_str(),
                "event": "failed",
                "stdout": String::from_utf8_lossy(stdout),
            })),
            TrFailedMsg(ref m) => self.write_message(&json!({
                "type": "test",
                "name": desc.name.as_str(),
                "event": "failed",
                "message": m,
            })),
            TrIgnored => self.write_message(&json!({
                "type": "test",
                "name": desc.name.as_str(),
                "event": "ignored",
            })),
            TrAllowedFail => self.write_message(&json!({
                "type": "test",
                "name": desc.name.as_str(),
                "event": "allowed_failure",
            })),
            TrBench(ref bs) => {
                let median = bs.ns_iter_summ.median as usize;
                let deviation = (bs.ns_iter_summ.max - bs.ns_iter_summ.min) as usize;

                let mbps = if bs.mb_s == 0 {
                    "".into()
                } else {
                    format!(r#", "mib_per_second": {}"#, bs.mb_s)
                };
                self.write_message(&json!({
                    "type": "bench",
                    "name": desc.name.as_str(),
                    "median": median,
                    "deviation": format!("{}{}", deviation, mbps),
                }))
            }
        }
    }

    fn write_timeout(&mut self, desc: &TestDesc) -> io::Result<()> {
        self.write_message(&json!({
            "type": "test",
            "name": desc.name.as_str(),
            "event": "timeout",
        }))
    }

    fn write_run_finish(&mut self, state: &ConsoleTestState) -> io::Result<bool> {
        self.write_message(&json!({
            "type": "suite",
            "event": if state.failed == 0 { "ok" } else { "failed" },
            "passed": state.passed,
            "failed": state.failed + state.allowed_fail,
            "allowed_fail": state.allowed_fail,
            "ignored": state.ignored,
            "measured": state.measured,
            "filtered_out": state.filtered_out,
        }))?;

        Ok(state.failed == 0)
    }
}
