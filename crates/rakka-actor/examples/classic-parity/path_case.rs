//! `path-from-string` command: renders `ActorPath.fromString` like the oracle does.

use rakka_actor::actor::ActorPath;

use crate::json::{write_str, write_text};

pub fn path_from_string(s: &str) -> String {
    let mut out = String::new();
    match ActorPath::from_string(s) {
        Ok(p) => {
            let a = p.address();
            out.push_str("{\"ok\":true,\"toString\":");
            write_text(&mut out, &p.to_string());
            out.push_str(",\"name\":");
            write_text(&mut out, p.name());
            out.push_str(",\"elements\":[");
            for (i, e) in p.elements().iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                write_text(&mut out, e);
            }
            out.push_str(&format!("],\"uid\":{}", p.uid()));
            out.push_str(",\"toStringWithoutAddress\":");
            write_text(&mut out, &p.to_string_without_address());
            out.push_str(",\"toSerializationFormat\":");
            write_text(&mut out, &p.to_serialization_format());
            out.push_str(",\"address\":{\"protocol\":");
            write_text(&mut out, a.protocol());
            out.push_str(",\"system\":");
            write_text(&mut out, a.system());
            out.push_str(",\"host\":");
            match a.host() {
                Some(h) => write_text(&mut out, h),
                None => out.push_str("null"),
            }
            out.push_str(",\"port\":");
            match a.port() {
                Some(p) => out.push_str(&p.to_string()),
                None => out.push_str("null"),
            }
            out.push_str(",\"toString\":");
            write_text(&mut out, &a.to_string());
            out.push_str("}}");
        }
        Err(f) => {
            out.push_str("{\"ok\":false,\"exception\":");
            write_text(&mut out, qualified_class(f.java_class()));
            out.push_str(",\"message\":");
            write_str(&mut out, &f.message_utf16());
            out.push('}');
        }
    }
    out
}

fn qualified_class(simple: &str) -> &str {
    match simple {
        "MalformedURLException" => "java.net.MalformedURLException",
        "NumberFormatException" => "java.lang.NumberFormatException",
        other => other,
    }
}
