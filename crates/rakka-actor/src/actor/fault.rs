// Derived from Apache Pekko 2.0.0 actor/src/main/scala/org/apache/pekko/actor/Actor.scala and actor/src/main/scala/org/apache/pekko/actor/dungeon/FaultHandling.scala (Apache-2.0).
//! Failures of actor code: the `Throwable` an actor "throws" and how the runtime reports it.
//! Rust has no checked exceptions, so an actor fails by unwinding with a [`Throwable`] payload
//! ([`throw`]) or with any ordinary panic, which is reported as a `java.lang.RuntimeException`.

use std::fmt;

/// A JVM exception as seen by Pekko: class name, optional message, optional cause and the superclasses deciders classify it by.
///
/// Classification is by subtype ([`is_a`](Throwable::is_a)). **A class that nobody declared anything about is an `Exception`**:
/// a user error needs no declaration. The superclasses are known only for the classes this runtime raises or names (see
/// `superclass_of`); for any other class, `java.lang` ones included, declare them with
/// [`with_superclasses`](Throwable::with_superclasses), e.g. `["java.lang.Error"]` for a throwable that is not an `Exception`.
///
/// A cause chain can be of any length: `Clone`, `PartialEq`, `Debug` and `Drop` walk it without a stack frame per cause. Because of
/// that `Drop`, a field cannot be moved out of a `Throwable` by value, by destructuring it by value or by struct update syntax
/// (error E0509): borrow it, or `take` it. And as `PartialEq` is not derived, a `const Throwable` cannot be used as a pattern.
pub struct Throwable {
    /// The JVM class name, e.g. `java.lang.IllegalStateException`.
    pub class: String,
    /// `getMessage`; `None` when the exception has no message.
    pub message: Option<String>,
    /// What [`with_superclasses`](Throwable::with_superclasses) declared, nearest first; empty when nothing was declared.
    superclasses: Vec<String>,
    /// `getCause`: what this exception wraps. Equality compares it too; [`Display`](fmt::Display), like `toString`, leaves it out.
    pub cause: Option<Box<Throwable>>,
}

/// The direct superclass of the classes this runtime raises or names, and of the two roots of the categories; `None` above
/// `java.lang.Throwable`. Not a model of the JDK: any other class is a direct subclass of `Exception`, the default category.
fn superclass_of(class: &str) -> Option<&'static str> {
    Some(match class {
        "java.lang.Throwable" => return None,
        "java.lang.Error"
        | "java.lang.Exception"
        | "org.apache.pekko.event.Logging$Error$NoCause$" => "java.lang.Throwable",
        "java.lang.IllegalStateException"
        | "java.lang.IllegalArgumentException"
        | "java.util.concurrent.RejectedExecutionException"
        | "org.apache.pekko.PekkoException" => "java.lang.RuntimeException",
        "java.lang.reflect.InvocationTargetException" => "java.lang.ReflectiveOperationException",
        "org.apache.pekko.ConfigurationException"
        | "org.apache.pekko.actor.ActorInitializationException"
        | "org.apache.pekko.actor.ActorKilledException"
        | "org.apache.pekko.actor.DeathPactException"
        | "org.apache.pekko.actor.InvalidActorNameException" => "org.apache.pekko.PekkoException",
        "org.apache.pekko.actor.PreRestartException"
        | "org.apache.pekko.actor.PostRestartException" => {
            "org.apache.pekko.actor.ActorInitializationException"
        }
        _ => "java.lang.Exception",
    })
}

impl Throwable {
    /// A throwable of `class` with an optional `message`.
    pub fn new(class: impl Into<String>, message: Option<&str>) -> Self {
        Throwable {
            class: class.into(),
            message: message.map(str::to_string),
            superclasses: Vec::new(),
            cause: None,
        }
    }

    /// The same exception caused by `cause` (the constructor argument of the same name on the JVM).
    pub fn with_cause(mut self, cause: Throwable) -> Self {
        self.cause = Some(Box::new(cause));
        self
    }

    /// Declares the superclasses of this class, nearest first; what the last one extends follows from it, by the same rules as
    /// for a class (an unknown one is an `Exception`). Replaces an earlier declaration; an empty one declares nothing.
    pub fn with_superclasses<S: Into<String>>(
        mut self,
        superclasses: impl IntoIterator<Item = S>,
    ) -> Self {
        self.superclasses = superclasses.into_iter().map(Into::into).collect();
        self
    }

    /// `isInstanceOf`: whether `class` (a fully qualified name) is the class of this throwable or one of its superclasses.
    pub fn is_a(&self, class: &str) -> bool {
        let furthest = self.superclasses.last().unwrap_or(&self.class);
        let inherited = std::iter::successors(superclass_of(furthest), |c| superclass_of(c));
        std::iter::once(&self.class)
            .chain(&self.superclasses)
            .map(String::as_str)
            .chain(inherited)
            .any(|own| own == class)
    }

    /// `new IllegalStateException(message)`.
    pub fn illegal_state(message: &str) -> Self {
        Throwable::new("java.lang.IllegalStateException", Some(message))
    }

    /// This throwable, its cause, the cause of that one and so on.
    fn chain(&self) -> impl Iterator<Item = &Throwable> {
        std::iter::successors(Some(self), |link| link.cause.as_deref())
    }

    /// A copy of this link alone, caused by `cause`.
    fn link_caused_by(&self, cause: Option<Box<Throwable>>) -> Throwable {
        Throwable {
            class: self.class.clone(),
            message: self.message.clone(),
            superclasses: self.superclasses.clone(),
            cause,
        }
    }

    /// `{:#?}` as `#[derive(Debug)]` writes it: each cause sits eight spaces to the right of what it causes.
    fn fmt_indented(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (mut link, mut depth) = (self, 0);
        loop {
            // no `..`: a new field must be printed here, so it does not compile until it is
            let Throwable {
                class,
                message,
                superclasses,
                cause,
            } = link;
            let pad = 8 * depth;
            f.write_str("Throwable {\n")?;
            indent(f, pad)?;
            writeln!(f, "    class: {class:?},")?;
            indent(f, pad)?;
            match message {
                Some(message) => {
                    f.write_str("    message: Some(\n")?;
                    indent(f, pad)?;
                    writeln!(f, "        {message:?},")?;
                    indent(f, pad)?;
                    f.write_str("    ),\n")?;
                }
                None => f.write_str("    message: None,\n")?,
            }
            indent(f, pad)?;
            if superclasses.is_empty() {
                f.write_str("    superclasses: [],\n")?;
            } else {
                f.write_str("    superclasses: [\n")?;
                for superclass in superclasses {
                    indent(f, pad)?;
                    writeln!(f, "        {superclass:?},")?;
                }
                indent(f, pad)?;
                f.write_str("    ],\n")?;
            }
            indent(f, pad)?;
            let Some(cause) = cause else {
                f.write_str("    cause: None,\n")?;
                break;
            };
            f.write_str("    cause: Some(\n")?;
            indent(f, pad + 8)?;
            (link, depth) = (cause, depth + 1);
        }
        for outer in (0..depth).rev() {
            indent(f, 8 * outer + 8)?;
            f.write_str("},\n")?;
            indent(f, 8 * outer + 4)?;
            f.write_str("),\n")?;
        }
        f.write_str("}")
    }
}

/// `width` spaces. A format width cannot do it: it is limited to `u16::MAX`.
fn indent(f: &mut fmt::Formatter<'_>, mut width: usize) -> fmt::Result {
    const SPACES: &str = "                                                                ";
    while width > 0 {
        let some = width.min(SPACES.len());
        f.write_str(&SPACES[..some])?;
        width -= some;
    }
    Ok(())
}

impl Drop for Throwable {
    /// Unlinks the causes one by one: each is dropped when it no longer has a cause of its own.
    fn drop(&mut self) {
        let mut next = self.cause.take();
        while let Some(mut link) = next {
            next = link.cause.take();
        }
    }
}

impl Clone for Throwable {
    fn clone(&self) -> Self {
        // built from the innermost cause outwards
        let causes: Vec<&Throwable> = self.chain().skip(1).collect();
        let cause = causes.into_iter().rev().fold(None, |cause, link| {
            Some(Box::new(link.link_caused_by(cause)))
        });
        self.link_caused_by(cause)
    }
}

impl PartialEq for Throwable {
    /// Chains of the same length whose classes, messages and declared superclasses match link by link.
    fn eq(&self, other: &Self) -> bool {
        // no `..`: a new field must be compared here, so it does not compile until it is. The causes are the rest of the chain.
        fn own_fields(link: &Throwable) -> (&String, &Option<String>, &Vec<String>) {
            let Throwable {
                class,
                message,
                superclasses,
                cause: _,
            } = link;
            (class, message, superclasses)
        }
        let own = self.chain().map(own_fields);
        own.eq(other.chain().map(own_fields))
    }
}

impl Eq for Throwable {}

impl fmt::Debug for Throwable {
    /// The text of `#[derive(Debug)]`.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if f.alternate() {
            return self.fmt_indented(f);
        }
        let mut open_causes = 0;
        for link in self.chain() {
            let Throwable {
                class,
                message,
                superclasses,
                cause,
            } = link; // no `..`, as in `fmt_indented`
            write!(
                f,
                "Throwable {{ class: {class:?}, message: {message:?}, superclasses: {superclasses:?}, cause: "
            )?;
            if cause.is_some() {
                f.write_str("Some(")?;
                open_causes += 1;
            }
        }
        f.write_str("None }")?;
        for _ in 0..open_causes {
            f.write_str(") }")?;
        }
        Ok(())
    }
}

impl fmt::Display for Throwable {
    /// `reason.getClass.getName` plus `": message"` when there is one.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.message {
            Some(m) => write!(f, "{}: {m}", self.class),
            None => f.write_str(&self.class),
        }
    }
}

/// `throw t`: unwinds the current actor callback without invoking the panic hook (nothing is printed).
pub fn throw(t: Throwable) -> ! {
    std::panic::resume_unwind(Box::new(t))
}

#[cfg(test)]
mod tests {
    use super::Throwable;

    /// The standard derives on a type of the same shape and name: the output users have been seeing.
    mod derived {
        #[allow(dead_code)] // read only by the derived `Debug`
        #[derive(Debug)]
        pub struct Throwable {
            pub class: String,
            pub message: Option<String>,
            pub superclasses: Vec<String>,
            pub cause: Option<Box<Throwable>>,
        }
    }

    fn as_derived(t: &Throwable) -> derived::Throwable {
        derived::Throwable {
            class: t.class.clone(),
            message: t.message.clone(),
            superclasses: t.superclasses.clone(),
            cause: t.cause.as_deref().map(|c| Box::new(as_derived(c))),
        }
    }

    fn three_links() -> Throwable {
        Throwable::new("a.B", Some("say \"x\"\n")).with_cause(
            Throwable::new("c.D", None)
                .with_superclasses(["g.H", "java.lang.Error"])
                .with_cause(Throwable::new("e.F", Some(""))),
        )
    }

    #[test]
    fn debug_matches_the_standard_derive_at_any_depth_and_indentation() {
        for t in [Throwable::new("A", None), three_links()] {
            let mirror = as_derived(&t);
            assert_eq!(format!("{t:?}"), format!("{mirror:?}"));
            assert_eq!(format!("{t:#?}"), format!("{mirror:#?}"));
            assert_eq!(format!("{t:>40.2?}"), format!("{mirror:>40.2?}"));
            // inside another value, which indents what it holds
            assert_eq!(
                format!("{:#?}", [Some(&t)]),
                format!("{:#?}", [Some(&mirror)])
            );
        }
    }

    #[test]
    fn debug_stops_at_the_first_error_of_the_writer() {
        use std::fmt::Write;
        /// Accepts `left` bytes, then fails and counts how often it is asked again.
        struct Short {
            left: usize,
            refused: usize,
        }
        impl Write for Short {
            fn write_str(&mut self, text: &str) -> std::fmt::Result {
                match self.left.checked_sub(text.len()) {
                    Some(left) if self.refused == 0 => {
                        self.left = left;
                        Ok(())
                    }
                    _ => {
                        self.refused += 1;
                        Err(std::fmt::Error)
                    }
                }
            }
        }
        let t = three_links();
        for alternate in [false, true] {
            let whole = if alternate {
                format!("{t:#?}")
            } else {
                format!("{t:?}")
            };
            for left in 0..whole.len() {
                let mut short = Short { left, refused: 0 };
                let written = if alternate {
                    write!(short, "{t:#?}")
                } else {
                    write!(short, "{t:?}")
                };
                assert!(written.is_err(), "{left}");
                assert_eq!(short.refused, 1, "{left}");
            }
        }
    }

    #[test]
    fn equality_compares_every_link_and_a_clone_is_equal_but_separate() {
        let t = three_links();
        let mut copy = t.clone();
        assert_eq!(copy, t);
        let innermost = copy.cause.as_mut().unwrap().cause.as_mut().unwrap();
        innermost.message = None;
        assert_ne!(copy, t);
        assert_eq!(t, three_links());

        let mut other_class = three_links();
        other_class.class = "z".into();
        assert_ne!(other_class, t);

        let mut shorter = three_links();
        shorter.cause.as_mut().unwrap().cause = None;
        assert_ne!(shorter, t);
        assert_ne!(t, shorter);
    }

    #[test]
    fn an_undeclared_class_is_an_exception_and_the_classes_of_the_runtime_keep_their_parents() {
        let user = Throwable::new("com.example.Failure", None);
        assert!(user.is_a("com.example.Failure"));
        assert!(user.is_a("java.lang.Exception") && user.is_a("java.lang.Throwable"));
        assert!(!user.is_a("java.lang.RuntimeException") && !user.is_a("java.lang.Error"));

        let state = Throwable::illegal_state("x");
        for parent in [
            "java.lang.RuntimeException",
            "java.lang.Exception",
            "java.lang.Throwable",
        ] {
            assert!(state.is_a(parent), "{parent}");
        }
        assert!(!state.is_a("java.lang.IllegalArgumentException"));

        for root in ["java.lang.Error", "java.lang.Throwable"] {
            let thrown = Throwable::new(root, None);
            assert!(thrown.is_a("java.lang.Throwable"), "{root}");
            assert!(!thrown.is_a("java.lang.Exception"), "{root}");
        }

        let post = Throwable::new("org.apache.pekko.actor.PostRestartException", None);
        for parent in [
            "org.apache.pekko.actor.ActorInitializationException",
            "org.apache.pekko.PekkoException",
            "java.lang.RuntimeException",
        ] {
            assert!(post.is_a(parent), "{parent}");
        }
    }

    #[test]
    fn declared_superclasses_replace_the_default_and_continue_from_the_last_one() {
        let nested = Throwable::new("a.Child", None)
            .with_superclasses(["a.Parent", "java.lang.IllegalArgumentException"]);
        for parent in [
            "a.Child",
            "a.Parent",
            "java.lang.IllegalArgumentException",
            "java.lang.RuntimeException",
            "java.lang.Exception",
        ] {
            assert!(nested.is_a(parent), "{parent}");
        }
        assert!(!nested.is_a("a.Other"));

        let error =
            Throwable::new("java.lang.AssertionError", None).with_superclasses(["java.lang.Error"]);
        assert!(error.is_a("java.lang.Error") && !error.is_a("java.lang.Exception"));

        // a user superclass that says nothing more is itself an exception
        let open = Throwable::new("a.Child", None).with_superclasses(["a.Parent"]);
        assert!(open.is_a("java.lang.Exception"));

        // declaring nothing leaves the default of the class
        let none = Throwable::new("java.lang.Error", None).with_superclasses::<&str>([]);
        assert!(!none.is_a("java.lang.Exception"));
    }

    #[test]
    fn declared_superclasses_survive_a_clone_and_take_part_in_equality_and_debug() {
        let plain = Throwable::new("a.Child", None);
        let declared = plain.clone().with_superclasses(["java.lang.Error"]);
        assert_ne!(declared, plain);
        assert_eq!(declared.clone(), declared);
        assert!(declared.clone().is_a("java.lang.Error"));
        assert!(format!("{declared:?}").contains(r#"superclasses: ["java.lang.Error"]"#));
        // the declaration is not part of the text, as on the JVM
        assert_eq!(declared.to_string(), plain.to_string());
        let outer = Throwable::illegal_state("outer").with_cause(declared);
        let cause = outer.clone().cause.take().unwrap();
        assert!(cause.is_a("java.lang.Error"));
    }

    /// `Throwable.toString` leaves the cause out; equality of the Rust value does not.
    #[test]
    fn a_cause_changes_equality_but_not_the_text() {
        let plain = Throwable::illegal_state("outer");
        let caused = plain.clone().with_cause(Throwable::illegal_state("inner"));
        assert_eq!(caused.to_string(), "java.lang.IllegalStateException: outer");
        assert_ne!(caused, plain);
        assert_eq!(
            caused.cause.as_deref(),
            Some(&Throwable::illegal_state("inner"))
        );
    }
}
