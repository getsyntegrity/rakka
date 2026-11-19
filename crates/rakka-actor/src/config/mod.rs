// Derived from Apache Pekko 2.0.0 actor/src/main/scala/org/apache/pekko/util/Helpers.scala and actor/src/main/scala/org/apache/pekko/actor/ActorSystem.scala (Apache-2.0).
// Original implementation: the HOCON behaviour was measured against Typesafe Config 1.4.9 (Apache-2.0, the library Pekko uses) with the oracle; none of its code was copied.
//! HOCON configuration (Q-X-1 (A): an own parser): `Config` with `withFallback`, late-bound substitutions, typed getters, durations and memory sizes.

mod error;
mod exact_decimal;
mod model;
mod parse;
mod reference;
mod resolve;
mod settings;
mod units;
mod validate;

#[cfg(test)]
mod tests;

use std::path::Path;

pub use error::{ConfigError, ConfigErrorKind};
pub use model::Number;
pub use parse::Resources;
pub use reference::{ACTOR_REFERENCE_CONF, TYPED_REFERENCE_CONF};
pub use settings::{
    DEFAULT_LOGGER, DEFAULT_LOGGING_FILTER, SLF4J_LOGGER, SLF4J_LOGGING_FILTER, Settings,
    SettingsError, SuspendDuration, amend_slf4j_config, normalize_extension_name,
};

use model::{Kind, Obj, Val, join_path, merge_obj, quote};

use crate::ConfigurationException;
use crate::util::{TimeUnit, Timeout};

/// `ConfigValueType`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ValueType {
    Object,
    List,
    Number,
    Boolean,
    Null,
    String,
}

impl ValueType {
    /// The name of the Java enum constant.
    pub fn name(self) -> &'static str {
        match self {
            ValueType::Object => "OBJECT",
            ValueType::List => "LIST",
            ValueType::Number => "NUMBER",
            ValueType::Boolean => "BOOLEAN",
            ValueType::Null => "NULL",
            ValueType::String => "STRING",
        }
    }
}

fn type_of(v: &Val) -> &'static str {
    match v.kind {
        Kind::Null => "NULL",
        Kind::Bool(_) => "BOOLEAN",
        Kind::Num(_) => "NUMBER",
        Kind::Str(_) => "STRING",
        Kind::List(_) => "LIST",
        Kind::Obj(_) => "OBJECT",
        Kind::Concat(_) | Kind::Subst(_) => "UNRESOLVED",
    }
}

/// A value of a [`Config`].
#[derive(Clone, Debug, PartialEq)]
pub struct ConfigValue(Val);

impl ConfigValue {
    /// `valueType`.
    pub fn value_type(&self) -> ValueType {
        match self.0.kind {
            Kind::Null => ValueType::Null,
            Kind::Bool(_) => ValueType::Boolean,
            Kind::Num(_) => ValueType::Number,
            Kind::List(_) => ValueType::List,
            Kind::Obj(_) => ValueType::Object,
            _ => ValueType::String,
        }
    }

    /// `render(ConfigRenderOptions.concise())`: compact JSON-like text.
    pub fn render_concise(&self) -> String {
        render_concise(&self.0)
    }
}

pub(crate) fn render_concise(v: &Val) -> String {
    match &v.kind {
        Kind::Null => "null".to_owned(),
        Kind::Bool(b) => b.to_string(),
        Kind::Num(n) => n.render(),
        Kind::Str(s) => quote(s),
        Kind::List(l) => format!(
            "[{}]",
            l.iter().map(render_concise).collect::<Vec<_>>().join(",")
        ),
        Kind::Obj(m) => {
            let fields: Vec<String> = m
                .iter()
                .filter_map(|(k, st)| {
                    st.first()
                        .map(|x| format!("{}:{}", quote(k), render_concise(x)))
                })
                .collect();
            format!("{{{}}}", fields.join(","))
        }
        Kind::Concat(_) | Kind::Subst(_) => "\"<unresolved>\"".to_owned(),
    }
}

/// An immutable HOCON configuration.
#[derive(Clone, Debug, PartialEq)]
pub struct Config {
    root: Obj,
    origin: String,
}

fn wrong_type(v: &Val, path: &str, expected: &str) -> ConfigError {
    ConfigError::new(
        ConfigErrorKind::WrongType,
        format!(
            "String: {}: {path} has type {} rather than {expected}",
            v.line,
            type_of(v)
        ),
    )
}

fn null_error(v: &Val, path: &str, expected: &str) -> ConfigError {
    ConfigError::new(
        ConfigErrorKind::Null,
        format!(
            "String: {}: Configuration key '{path}' is set to null but expected {expected}",
            v.line
        ),
    )
}

fn bad_value(v: &Val, path: &str, msg: &str) -> ConfigError {
    ConfigError::new(
        ConfigErrorKind::BadValue,
        format!("String: {}: Invalid value at '{path}': {msg}", v.line),
    )
}

impl Config {
    /// An empty configuration.
    pub fn empty() -> Config {
        Config {
            root: Obj::new(),
            origin: "String".to_owned(),
        }
    }

    /// `ConfigFactory.parseString`: no `include` resolves (a plain include is ignored, a `required` one fails).
    pub fn parse_string(text: &str) -> Result<Config, ConfigError> {
        Config::parse_string_with(text, &[])
    }

    /// `parseString` where `include "name"` finds the given resources.
    pub fn parse_string_with(text: &str, resources: Resources<'_>) -> Result<Config, ConfigError> {
        let root = parse::parse(text, resources)?;
        let lines = text.matches('\n').count() + 1;
        let origin = if lines > 1 {
            format!("String: 1-{lines}")
        } else {
            "String: 1".to_owned()
        };
        Ok(Config { root, origin })
    }

    /// `this.withFallback(other)`: this configuration wins, objects merge key by key.
    #[must_use]
    pub fn with_fallback(&self, other: &Config) -> Config {
        Config {
            root: merge_obj(&self.root, &other.root),
            origin: self.origin.clone(),
        }
    }

    /// `resolve()`: substitutions are replaced by the values they point to.
    pub fn resolve(&self) -> Result<Config, ConfigError> {
        Ok(Config {
            root: resolve::resolve_root(&self.root)?,
            origin: self.origin.clone(),
        })
    }

    /// The Pekko reference configuration (`pekko-actor` `reference.conf`, resolved), with `pekko.version` = 2.0.0 (Q-CFG-5).
    pub fn default_reference() -> Config {
        Config::parse_string_with(
            ACTOR_REFERENCE_CONF,
            &[("version", reference::VERSION_CONF)],
        )
        .and_then(|c| c.resolve())
        .expect("the embedded reference.conf is valid")
    }

    /// What `ConfigFactory.defaultReference()` gives when `pekko-actor-typed` is on the classpath: its `reference.conf`
    /// over the one of `pekko-actor`, resolved.
    #[doc(hidden)]
    pub fn default_reference_with_typed() -> Config {
        let actor = Config::parse_string_with(
            ACTOR_REFERENCE_CONF,
            &[("version", reference::VERSION_CONF)],
        )
        .expect("the embedded reference.conf is valid");
        Config::parse_string(TYPED_REFERENCE_CONF)
            .expect("the embedded typed reference.conf is valid")
            .with_fallback(&actor)
            .resolve()
            .expect("the embedded reference.conf files resolve")
    }

    /// `ConfigFactory.load()` with the sources of Q-CFG-6 (a): the application file (default `application.conf` in the
    /// working directory; a missing default is an empty configuration, a missing explicit one is an error) over the
    /// embedded reference configuration. There is no classpath scanning.
    pub fn load(application: Option<&Path>) -> Result<Config, ConfigError> {
        let app = match application {
            Some(path) => {
                let text = std::fs::read_to_string(path).map_err(|e| {
                    ConfigError::new(ConfigErrorKind::Io, format!("{}: {e}", path.display()))
                })?;
                Config::parse_string(&text)?
            }
            None => match std::fs::read_to_string("application.conf") {
                Ok(text) => Config::parse_string(&text)?,
                Err(_) => Config::empty(),
            },
        };
        app.with_fallback(&Config::default_reference()).resolve()
    }

    fn missing(&self, path: &str) -> ConfigError {
        ConfigError::new(
            ConfigErrorKind::Missing,
            format!(
                "{}: No configuration setting found for key '{path}'",
                self.origin
            ),
        )
    }

    /// The effective value at `segments`; `None` if absent.
    fn find(&self, segments: &[String], shown: &str) -> Result<Option<Val>, ConfigError> {
        let not_resolved = || {
            ConfigError::new(
                ConfigErrorKind::NotResolved,
                format!(
                    "{shown} has not been resolved, you need to call Config#resolve(), see API docs for Config#resolve()"
                ),
            )
        };
        let mut cur: Vec<Val> = vec![Val::new(1, Kind::Obj(self.root.clone()))];
        for key in segments {
            let mut next: Vec<Val> = Vec::new();
            for it in &cur {
                match &it.kind {
                    Kind::Obj(m) => {
                        if let Some(st) = m.get(key) {
                            next.extend(st.iter().cloned());
                        }
                    }
                    Kind::Concat(_) | Kind::Subst(_) => return Err(not_resolved()),
                    _ => break,
                }
            }
            if next.is_empty() {
                return Ok(None);
            }
            cur = next;
        }
        match cur.first() {
            None => Ok(None),
            Some(top) => match (&top.kind, cur.get(1).map(|v| &v.kind)) {
                (Kind::Concat(_) | Kind::Subst(_), _) => Err(not_resolved()),
                (Kind::Obj(_), Some(Kind::Concat(_) | Kind::Subst(_))) => Err(not_resolved()),
                _ => Ok(Some(top.clone())),
            },
        }
    }

    fn segments(path: &str) -> Result<Vec<String>, ConfigError> {
        parse::parse_path(path).map_err(|m| ConfigError::new(ConfigErrorKind::BadPath, m))
    }

    /// The value at `path`, failing with `Missing` or `Null` like the typed getters do.
    fn get_val(&self, path: &str, expected: &str) -> Result<Val, ConfigError> {
        let segs = Config::segments(path)?;
        match self.find(&segs, path)? {
            None => Err(self.missing(path)),
            Some(v) if v.kind == Kind::Null => Err(null_error(&v, path, expected)),
            Some(v) => Ok(v),
        }
    }

    /// `hasPath`: the path is set to a value that is not null.
    pub fn has_path(&self, path: &str) -> Result<bool, ConfigError> {
        let segs = Config::segments(path)?;
        Ok(matches!(self.find(&segs, path)?, Some(v) if v.kind != Kind::Null))
    }

    /// `hasPathOrNull`.
    pub fn has_path_or_null(&self, path: &str) -> Result<bool, ConfigError> {
        let segs = Config::segments(path)?;
        Ok(self.find(&segs, path)?.is_some())
    }

    /// `getValue`.
    pub fn get_value(&self, path: &str) -> Result<ConfigValue, ConfigError> {
        let segs = Config::segments(path)?;
        self.find(&segs, path)?
            .map(ConfigValue)
            .ok_or_else(|| self.missing(path))
    }

    /// The keys of the root object, sorted.
    pub fn root_keys(&self) -> Vec<String> {
        self.root.keys().cloned().collect()
    }

    /// Every leaf (a value that is not an object) with its path expression, sorted by path: what `entrySet` lists plus nulls.
    pub fn leaves(&self) -> Vec<(String, ConfigValue)> {
        fn walk(
            o: &Obj,
            prefix: &mut Vec<String>,
            out: &mut std::collections::BTreeMap<String, ConfigValue>,
        ) {
            for (k, st) in o {
                let Some(v) = st.first() else { continue };
                prefix.push(k.clone());
                match &v.kind {
                    Kind::Obj(m) => walk(m, prefix, out),
                    _ => {
                        out.insert(join_path(prefix), ConfigValue(v.clone()));
                    }
                }
                prefix.pop();
            }
        }
        let mut out = std::collections::BTreeMap::new();
        walk(&self.root, &mut Vec::new(), &mut out);
        out.into_iter().collect()
    }

    /// `getString`.
    pub fn get_string(&self, path: &str) -> Result<String, ConfigError> {
        let v = self.get_val(path, "STRING")?;
        scalar_text(&v).ok_or_else(|| wrong_type(&v, path, "STRING"))
    }

    /// `getInt`.
    pub fn get_int(&self, path: &str) -> Result<i32, ConfigError> {
        let v = self.get_val(path, "NUMBER")?;
        let n = self.number_of(&v, path)?;
        let long = match n.repr {
            model::NumRepr::Int(i) => i,
            model::NumRepr::Float(f) => f as i64,
        };
        i32::try_from(long).map_err(|_| {
            ConfigError::new(
                ConfigErrorKind::WrongType,
                format!(
                    "String: {}: {path} has type out-of-range value {} rather than 32-bit integer",
                    v.line,
                    n.text()
                ),
            )
        })
    }

    /// `getLong`.
    pub fn get_long(&self, path: &str) -> Result<i64, ConfigError> {
        let v = self.get_val(path, "NUMBER")?;
        let n = self.number_of(&v, path)?;
        Ok(match n.repr {
            model::NumRepr::Int(i) => i,
            model::NumRepr::Float(f) => f as i64,
        })
    }

    /// `getDouble`.
    pub fn get_double(&self, path: &str) -> Result<f64, ConfigError> {
        let v = self.get_val(path, "NUMBER")?;
        Ok(self.number_of(&v, path)?.as_f64())
    }

    fn number_of(&self, v: &Val, path: &str) -> Result<Number, ConfigError> {
        match &v.kind {
            Kind::Num(n) => Ok(n.clone()),
            Kind::Str(s) => Number::parse(s.trim()).ok_or_else(|| wrong_type(v, path, "NUMBER")),
            _ => Err(wrong_type(v, path, "NUMBER")),
        }
    }

    /// `getBoolean`: `true`/`yes`/`on` and `false`/`no`/`off`, quoted or not.
    pub fn get_boolean(&self, path: &str) -> Result<bool, ConfigError> {
        let v = self.get_val(path, "BOOLEAN")?;
        match &v.kind {
            Kind::Bool(b) => Ok(*b),
            Kind::Str(s) if matches!(s.as_str(), "true" | "yes" | "on") => Ok(true),
            Kind::Str(s) if matches!(s.as_str(), "false" | "no" | "off") => Ok(false),
            _ => Err(wrong_type(&v, path, "BOOLEAN")),
        }
    }

    /// `getDuration(path, unit)`, truncating toward zero; a number without unit is milliseconds.
    pub fn get_duration(&self, path: &str, unit: TimeUnit) -> Result<i64, ConfigError> {
        let v = self.get_val(path, "DURATION")?;
        let text = scalar_text(&v).ok_or_else(|| wrong_type(&v, path, "STRING"))?;
        units::parse_duration(&text, unit).map_err(|m| bad_value(&v, path, &m))
    }

    /// `getBytes`.
    pub fn get_bytes(&self, path: &str) -> Result<i64, ConfigError> {
        let v = self.get_val(path, "MEMORY SIZE")?;
        let text = scalar_text(&v).ok_or_else(|| wrong_type(&v, path, "STRING"))?;
        units::parse_size(&text).map_err(|m| bad_value(&v, path, &m))
    }

    /// `getMemorySize(path).toBytes`.
    pub fn get_memory_size(&self, path: &str) -> Result<i64, ConfigError> {
        self.get_bytes(path)
    }

    /// `ConfigOps.getMillisDuration` (AC-UTIL-002.4): whole milliseconds as a duration.
    pub fn get_millis_duration(&self, path: &str) -> Result<Timeout, ConfigError> {
        let ms = self.get_duration(path, TimeUnit::Milliseconds)?;
        Timeout::of(ms, TimeUnit::Milliseconds)
            .map_err(|e| duration_out_of_range(path, e.message()))
    }

    /// `ConfigOps.getNanosDuration` (AC-UTIL-002.4): whole nanoseconds as a duration.
    pub fn get_nanos_duration(&self, path: &str) -> Result<Timeout, ConfigError> {
        let ns = self.get_duration(path, TimeUnit::Nanoseconds)?;
        Timeout::of(ns, TimeUnit::Nanoseconds).map_err(|e| duration_out_of_range(path, e.message()))
    }

    fn list_vals(&self, path: &str) -> Result<Vec<Val>, ConfigError> {
        let v = self.get_val(path, "LIST")?;
        match &v.kind {
            Kind::List(l) => Ok(l.clone()),
            Kind::Obj(m) => {
                let mut keyed: Vec<(i64, &Val)> = Vec::new();
                for (k, st) in m {
                    match (k.parse::<i64>(), st.first()) {
                        (Ok(n), Some(x)) if n >= 0 => keyed.push((n, x)),
                        _ => {}
                    }
                }
                if keyed.is_empty() {
                    return Err(wrong_type(&v, path, "LIST"));
                }
                keyed.sort_by_key(|(n, _)| *n);
                Ok(keyed.into_iter().map(|(_, x)| x.clone()).collect())
            }
            _ => Err(wrong_type(&v, path, "LIST")),
        }
    }

    fn list_error(v: &Val, path: &str, expected: &str) -> ConfigError {
        ConfigError::new(
            ConfigErrorKind::WrongType,
            format!(
                "String: {}: {path} has type list of {} rather than list of {expected}",
                v.line,
                type_of(v)
            ),
        )
    }

    /// `getStringList`.
    pub fn get_string_list(&self, path: &str) -> Result<Vec<String>, ConfigError> {
        self.list_vals(path)?
            .iter()
            .map(|v| {
                scalar_text(v)
                    .filter(|_| v.kind != Kind::Null)
                    .ok_or_else(|| Config::list_error(v, path, "STRING"))
            })
            .collect()
    }

    /// `getIntList`.
    pub fn get_int_list(&self, path: &str) -> Result<Vec<i32>, ConfigError> {
        self.list_vals(path)?
            .iter()
            .map(|v| {
                let n = match &v.kind {
                    Kind::Num(n) => Some(n.clone()),
                    Kind::Str(s) => Number::parse(s.trim()),
                    _ => None,
                }
                .ok_or_else(|| Config::list_error(v, path, "NUMBER"))?;
                let long = match n.repr {
                    model::NumRepr::Int(i) => i,
                    model::NumRepr::Float(f) => f as i64,
                };
                i32::try_from(long).map_err(|_| Config::list_error(v, path, "32-bit integer"))
            })
            .collect()
    }

    /// `getConfig`.
    pub fn get_config(&self, path: &str) -> Result<Config, ConfigError> {
        let v = self.get_val(path, "OBJECT")?;
        match v.kind {
            Kind::Obj(root) => Ok(Config {
                root,
                origin: format!("String: {}", v.line),
            }),
            _ => Err(wrong_type(&v, path, "OBJECT")),
        }
    }

    /// `getConfigList`.
    pub fn get_config_list(&self, path: &str) -> Result<Vec<Config>, ConfigError> {
        self.list_vals(path)?
            .into_iter()
            .map(|v| match v.kind {
                Kind::Obj(root) => Ok(Config {
                    root,
                    origin: format!("String: {}", v.line),
                }),
                _ => Err(Config::list_error(&v, path, "OBJECT")),
            })
            .collect()
    }

    /// `checkValid(reference, restrictToPaths)` (AC-CFG-019, Q-CFG-3 (a)).
    pub fn check_valid(
        &self,
        reference: &Config,
        restrict_to_paths: &[&str],
    ) -> Result<(), ConfigError> {
        validate::check_valid(self, reference, restrict_to_paths)
    }

    /// `ConfigOps.renderWithRedactions` (AC-UTIL-002.5): the tree as HOCON text, the values of environment variables
    /// hidden and the user name shown as `<username>` (Q-CFG-4 (a): the text is not Typesafe's).
    pub fn render_with_redactions(&self) -> String {
        validate::render_with_redactions(&self.root)
    }

    /// `withoutPath`: the configuration without the key at `path`.
    pub fn without_path(&self, path: &str) -> Result<Config, ConfigError> {
        fn remove(o: &mut Obj, segs: &[String]) {
            let Some((first, rest)) = segs.split_first() else {
                return;
            };
            if rest.is_empty() {
                o.remove(first);
            } else if let Some(stack) = o.get_mut(first) {
                for v in stack.iter_mut() {
                    if let Kind::Obj(m) = &mut v.kind {
                        remove(m, rest);
                    }
                }
            }
        }
        let segs = Config::segments(path)?;
        let mut root = self.root.clone();
        remove(&mut root, &segs);
        Ok(Config {
            root,
            origin: self.origin.clone(),
        })
    }

    pub(crate) fn root_obj(&self) -> &Obj {
        &self.root
    }

    pub(crate) fn origin(&self) -> &str {
        &self.origin
    }
}

fn duration_out_of_range(path: &str, msg: &str) -> ConfigError {
    ConfigError::new(
        ConfigErrorKind::BadValue,
        format!("Invalid value at '{path}': {msg}"),
    )
}

fn scalar_text(v: &Val) -> Option<String> {
    match &v.kind {
        Kind::Str(s) => Some(s.clone()),
        Kind::Num(n) => Some(n.text().to_owned()),
        Kind::Bool(b) => Some(b.to_string()),
        _ => None,
    }
}

/// `ConfigOps`-style check of `pekko.version` against the version this crate reports (AC-CFG-003, Q-CFG-5 (a)).
pub fn check_pekko_version(config: &Config) -> Result<(), ConfigurationException> {
    let config_version = config
        .get_string("pekko.version")
        .map_err(|e| ConfigurationException::new(e.message()))?;
    if config_version == crate::pekko_version::CURRENT {
        Ok(())
    } else {
        Err(ConfigurationException::new(format!(
            "Pekko JAR version [{}] does not match the provided config version [{config_version}]",
            crate::pekko_version::CURRENT
        )))
    }
}
