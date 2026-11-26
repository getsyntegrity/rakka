"""Q-X-22 R2: adding only `pub mod x;` selects nothing for that file when the new module is proven unable to affect existing code."""
import json, os, subprocess, tempfile, unittest

import pr_selection

DISPATCH = "crates/rakka-actor/src/dispatch/"
PARENT, MODULE = DISPATCH + "mod.rs", DISPATCH + "new_queue.rs"
MUTANTS = {"parent-code": ("src/dispatch/mod.rs", "o", "n", "why"), "unit-sibling": ("src/dispatch/mailboxes.rs", "o", "n", "why"),
           "new-behaviour": ("src/dispatch/new_queue.rs", "o", "n", "why")}
UNITS = {"unit-sibling": ("dispatch::mailboxes", "a_test")}
BASE_PARENT = "pub mod mailboxes;\npub mod message_queue;\n"
OWN_ONLY = '''//! impl Display for String {} in a comment is not code
use crate::dispatch::message_queue::MessageQueue;

pub struct NewQueue<T> { items: Vec<T> }
enum State { Idle }

impl<T: Fn() -> u8> NewQueue<T> {
    pub fn push(&self, f: impl Fn()) -> impl Iterator<Item = u8> { vec![format!("impl Foo for Bar {{}}")].into_iter().map(|_| 1) }
}
impl<T> MessageQueue for NewQueue<T> where T: Send { fn clean(&self) { assert!(true); } }
unsafe impl Send for State {}

#[cfg(test)]
mod tests {
    #[test]
    fn works() { assert_eq!('{', '{'); let _ = r#"impl X for Y"#; }
}
'''


def narrow(head, base=None, changed=None):
    base = {PARENT: BASE_PARENT, **(base or {})}
    head = {PARENT: BASE_PARENT + "pub mod new_queue;\n", MODULE: OWN_ONLY, **head}
    changed = changed or [path for path in head if head[path] != base.get(path)]
    return pr_selection.narrow(changed, MUTANTS, {}, UNITS, base.get, head.get, lambda prefix: sorted(p for p in head if p == prefix + ".rs" or p.startswith(prefix + "/")))


class PubModOnly(unittest.TestCase):
    def test_an_independent_new_module_does_not_select_the_mutants_of_its_parent_and_the_report_says_which_did_not_run(self):
        report = narrow({})
        self.assertEqual(report["selected"], ["new-behaviour"])
        self.assertEqual(report["rules"], ["Q-X-22 R2: " + PARENT])
        self.assertEqual(report["not_selected_by_rule"], ["parent-code", "unit-sibling"])

    def test_another_changed_file_of_the_same_change_still_selects_its_own_mutants(self):
        report = narrow({}, changed=[PARENT, MODULE, DISPATCH + "mailboxes.rs"])
        self.assertEqual(report["selected"], ["unit-sibling", "new-behaviour"])

    def test_a_line_inserted_under_an_existing_attribute_takes_it_over_and_keeps_the_conservative_selection(self):
        base = "#[cfg(test)]\nmod helpers;\n"
        self.assert_conservative({PARENT: "#[cfg(test)]\npub mod new_queue;\nmod helpers;\n"}, base={PARENT: base})

    def assert_conservative(self, head, base=None, why=""):
        report = narrow(head, base)
        self.assertIn("parent-code", report["selected"], why)
        self.assertIn("unit-sibling", report["selected"], why)
        self.assertEqual(report["rules"], [], why)

    def test_any_other_edit_of_the_parent_keeps_the_conservative_selection(self):
        for parent in (BASE_PARENT + "pub mod new_queue;\nfn extra() {}\n", "pub mod mailboxes;\npub mod new_queue;\n",
                       BASE_PARENT + "mod new_queue;\n", BASE_PARENT + "pub(crate) mod new_queue;\n", BASE_PARENT + "#[macro_use]\npub mod new_queue;\n",
                       BASE_PARENT + "pub mod new_queue;\npub use new_queue::NewQueue;\n"):
            self.assert_conservative({PARENT: parent}, why=parent)

    def test_a_module_that_already_existed_or_cannot_be_found_keeps_the_conservative_selection(self):
        self.assert_conservative({}, base={MODULE: "pub struct Old;\n"})
        self.assert_conservative({PARENT: BASE_PARENT + "pub mod absent;\n"})

    def test_anything_that_reaches_existing_code_keeps_the_conservative_selection(self):
        for code in ("impl Display for ActorRef {}", "impl<T> Describe for T {}", "impl From<NewQueue<u8>> for String {}", "impl dyn MessageQueue {}",
                     "impl crate::dispatch::Envelope { fn x(&self) {} }", "impl Clone for &Mailbox {}", "fn f() { impl Drop for Envelope { fn drop(&mut self) {} } }",
                     "register!(NewQueue);", "mod inner { delegate! { A } }", "macro_rules! m { () => {} }", "#[macro_export]\nstruct Z;",
                     "#[no_mangle]\npub fn hook() {}", "extern \"C\" { fn x(); }", "#[global_allocator]\nstatic A: Alloc = Alloc;",
                     "#[path = \"../actor/cell.rs\"]\nmod stolen;", "include!(\"x.rs\");"):
            self.assert_conservative({MODULE: OWN_ONLY + code + "\n"}, why=code)

    def test_a_type_of_the_same_name_elsewhere_never_makes_an_outside_type_local(self):
        shadow = "#[cfg(test)]\nmod shadow { struct T; struct Envelope; }\n"
        for code in ("impl<T> Tag for T {}", "use crate::dispatch::Envelope;\nimpl Drop for Envelope {}", "struct U;\nimpl<U> Tag for U {}"):
            self.assert_conservative({MODULE: OWN_ONLY + shadow + code + "\n"}, why=code)

    def test_a_hidden_parameter_a_cfg_twin_or_a_raw_attribute_name_keeps_the_conservative_selection(self):
        for code in ("struct T2;\nimpl<#[cfg(all())] T2> Tag for T2 {}", "#[r#ctor]\nfn boot() {}",
                     "#[cfg(test)]\nstruct Envelope;\n#[cfg(not(test))]\nuse crate::dispatch::Envelope;\nimpl Drop for Envelope {}"):
            self.assert_conservative({MODULE: OWN_ONLY + code + "\n"}, why=code)

    def test_unknown_macros_and_attributes_and_constructs_the_reader_cannot_follow_keep_the_conservative_selection(self):
        for code in ("::a::b! {}", "a::vec![1];", "r#m!();", "fn f() { register!(A); }", "const _: () = { fn g() {} };",
                     "#[cfg_attr(all(), no_mangle)]\npub fn hook() {}", "#[ctor::ctor]\nfn boot() {}", "#[derive(Clone, Serialize)]\nstruct S;",
                     "/* x /* y */ \" */\nimpl Drop for Envelope {}\n// \""):
            self.assert_conservative({MODULE: OWN_ONLY + code + "\n"}, why=code)

    def test_a_pub_mod_that_takes_over_an_attribute_or_lands_inside_a_comment_keeps_the_conservative_selection(self):
        for base, head in (("#[cfg(test)]\n// helpers\nmod helpers;\n", "#[cfg(test)]\n// helpers\npub mod new_queue;\nmod helpers;\n"),
                           ("#[cfg(\n    test\n)]\nmod helpers;\n", "#[cfg(\n    test\n)]\npub mod new_queue;\nmod helpers;\n"),
                           ("/*\n*/\n", "/*\npub mod new_queue;\n*/\n")):
            self.assert_conservative({PARENT: head}, base={PARENT: base}, why=head)

    def test_a_module_name_that_could_change_how_existing_paths_resolve_keeps_the_conservative_selection(self):
        for name, base in (("str", BASE_PARENT), ("core", BASE_PARENT), ("new_queue", BASE_PARENT + "use crate::new_queue;\n")):
            module = DISPATCH + name + ".rs"
            report = narrow({PARENT: base + f"pub mod {name};\n", module: "pub struct A;\n"}, base={PARENT: base})
            self.assertEqual(report["rules"], [], name)

    def test_every_file_of_a_directory_module_is_analysed(self):
        directory = DISPATCH + "new_queue/"
        self.assertEqual(narrow({MODULE: "mod part;\npub struct A;\n", directory + "part.rs": "impl super::A { }\n"})["rules"], [])
        self.assertEqual(len(narrow({MODULE: "mod part;\npub struct A;\n", directory + "part.rs": "pub struct B;\nimpl B {}\n"})["rules"]), 1)


class FromGit(unittest.TestCase):
    def setUp(self):
        directory = tempfile.TemporaryDirectory()
        self.addCleanup(directory.cleanup)
        self.repo = directory.name
        self.git("init", "-q")
        self.write(PARENT, BASE_PARENT)
        self.commit("base")
        self.git("branch", "base-ref")

    def git(self, *arguments):
        subprocess.run(["git", "-c", "user.name=t", "-c", "user.email=t@t", *arguments], cwd=self.repo, check=True, capture_output=True)

    def write(self, relative, text):
        path = os.path.join(self.repo, relative)
        os.makedirs(os.path.dirname(path), exist_ok=True)
        with open(path, "w") as out:
            out.write(text)

    def commit(self, message):
        self.git("add", "-A")
        self.git("commit", "-q", "-m", message)

    def test_the_base_is_read_from_git_and_the_report_is_kept_on_disk(self):
        self.write(PARENT, BASE_PARENT + "pub mod new_queue;\n")
        self.write(MODULE, OWN_ONLY)
        self.commit("slice")
        report = pr_selection.select_for_pr("base-ref", MUTANTS, {}, UNITS, repo=self.repo)
        self.assertEqual((report["selected"], report["rules"]), (["new-behaviour"], ["Q-X-22 R2: " + PARENT]))
        with open(os.path.join(self.repo, report["report_path"])) as saved:
            self.assertEqual(json.load(saved)["not_selected_by_rule"], ["parent-code", "unit-sibling"])


if __name__ == "__main__":
    unittest.main()
