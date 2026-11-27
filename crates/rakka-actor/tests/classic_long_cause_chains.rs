// Derived from Apache Pekko 2.0.0 actor/src/main/scala/org/apache/pekko/actor/Actor.scala (Apache-2.0).
//! A user can build a `Throwable` with any number of causes: comparing, cloning, printing and dropping it must not need a stack
//! frame per cause. A stack overflow aborts the process, so each scenario runs in a child: this test binary again.

use std::fmt::Write;
use std::mem::forget;
use std::process::Command;

use rakka_actor::actor::fault::Throwable;

const CHILD: &str = "RAKKA_LONG_CAUSE_CHAIN";
const SMALL_STACK: usize = 128 << 10;

/// Runs `scenario` on a small stack in a child process that executes only the test `test`.
fn in_a_child_on_a_small_stack(test: &str, scenario: fn()) {
    if std::env::var_os(CHILD).is_none() {
        let mut child = Command::new(std::env::current_exe().unwrap());
        let output = child.args(["--exact", test]).env(CHILD, "1").output();
        let output = output.unwrap();
        let errors = String::from_utf8_lossy(&output.stderr);
        assert!(output.status.success(), "{}: {errors}", output.status);
        // (a filter that matched nothing would also succeed)
        return assert!(String::from_utf8_lossy(&output.stdout).contains("1 passed"));
    }
    let thread = std::thread::Builder::new().stack_size(SMALL_STACK);
    thread.spawn(scenario).unwrap().join().unwrap();
}

/// `links` throwables, each caused by the next; the innermost one has the message `root`.
fn chain(links: usize, root: &str) -> Throwable {
    let mut chain = Throwable::new("E", Some(root));
    for _ in 1..links {
        chain = Throwable::new("E", None).with_cause(chain);
    }
    chain
}

/// The number of links of `chain` and the message of the innermost one, found without recursion.
fn measure(chain: &Throwable) -> (usize, Option<&str>) {
    let (mut links, mut link) = (1, chain);
    while let Some(cause) = &link.cause {
        (links, link) = (links + 1, cause);
    }
    (links, link.message.as_deref())
}

#[test]
fn a_very_long_cause_chain_is_dropped() {
    in_a_child_on_a_small_stack("a_very_long_cause_chain_is_dropped", || {
        drop(chain(100_000, "root"));
    });
}

#[test]
fn a_very_long_cause_chain_is_cloned() {
    in_a_child_on_a_small_stack("a_very_long_cause_chain_is_cloned", || {
        let original = chain(100_000, "root");
        let copy = original.clone();
        assert_eq!(measure(&copy), (100_000, Some("root")));
        forget((original, copy)); // dropping is the subject of another test
    });
}

#[test]
fn very_long_cause_chains_are_compared() {
    in_a_child_on_a_small_stack("very_long_cause_chains_are_compared", || {
        let (one, same) = (chain(100_000, "root"), chain(100_000, "root"));
        let (other_root, shorter) = (chain(100_000, "toor"), chain(99_999, "root"));
        assert!(one == same);
        assert!(one != other_root);
        assert!(one != shorter);
        assert!(shorter != one);
        forget((one, same, other_root, shorter));
    });
}

#[test]
fn a_very_long_cause_chain_is_formatted_for_debugging() {
    in_a_child_on_a_small_stack("a_very_long_cause_chain_is_formatted_for_debugging", || {
        let original = chain(20_000, "root");
        let text = format!("{original:?}");
        assert!(text.starts_with(
            r#"Throwable { class: "E", message: None, cause: Some(Throwable { class: "E", "#
        ));
        let end = format!(
            r#"message: Some("root"), cause: None }}{}"#,
            ") }".repeat(19_999)
        );
        assert!(text.ends_with(&end));
        assert_eq!(text.matches("Throwable {").count(), 20_000);
        forget(original);
    });
}

/// Keeps the size and the last bytes of what is written: the indented form grows with the square of the number of causes.
#[derive(Default)]
struct Tail {
    written: usize,
    last: String,
}

impl Write for Tail {
    fn write_str(&mut self, text: &str) -> std::fmt::Result {
        self.written += text.len();
        self.last.push_str(text);
        if self.last.len() > 4096 {
            self.last.drain(..self.last.len() - 1024);
        }
        Ok(())
    }
}

#[test]
fn a_long_cause_chain_is_formatted_for_debugging_with_indentation() {
    in_a_child_on_a_small_stack(
        "a_long_cause_chain_is_formatted_for_debugging_with_indentation",
        || {
            let original = chain(2_000, "root");
            let mut tail = Tail::default();
            write!(tail, "{original:#?}").unwrap();
            assert!(
                tail.last
                    .ends_with("\n                },\n            ),\n        },\n    ),\n}")
            );
            // each link below the first is indented eight spaces more than the one before
            assert!(tail.written > 8 * 6 * 2_000 * 1_999 / 2);
            forget(original);
        },
    );
}
