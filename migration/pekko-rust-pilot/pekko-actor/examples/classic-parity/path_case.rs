//! `path-from-string` command: renders `ActorPath.fromString` like the oracle does.

use pekko_actor::actor_path::ActorPath;
use crate::json::write_str;

pub fn path_from_string(s: &[u16]) -> String {
    let mut out = String::new();
    match ActorPath::from_string(s) {
        Ok(p) => {
            let a = &p.address;
            out.push_str("{\"ok\":true,\"toString\":");
            write_str(&mut out, &p.to_jstring());
            out.push_str(",\"name\":");
            write_str(&mut out, &p.name());
            out.push_str(",\"elements\":[");
            for (i, e) in p.elements().iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                write_str(&mut out, e);
            }
            out.push_str(&format!("],\"uid\":{}", p.uid));
            out.push_str(",\"toStringWithoutAddress\":");
            write_str(&mut out, &p.to_string_without_address());
            out.push_str(",\"toSerializationFormat\":");
            write_str(&mut out, &p.to_serialization_format());
            out.push_str(",\"address\":{\"protocol\":");
            write_str(&mut out, &a.protocol);
            out.push_str(",\"system\":");
            write_str(&mut out, &a.system);
            out.push_str(",\"host\":");
            match &a.host {
                Some(h) => write_str(&mut out, h),
                None => out.push_str("null"),
            }
            out.push_str(",\"port\":");
            match a.port {
                Some(p) => out.push_str(&p.to_string()),
                None => out.push_str("null"),
            }
            out.push_str(",\"toString\":");
            write_str(&mut out, &a.to_jstring());
            out.push_str("}}");
        }
        Err(f) => {
            out.push_str("{\"ok\":false,\"exception\":");
            write_str(&mut out, &f.exception.encode_utf16().collect::<Vec<_>>());
            out.push_str(",\"message\":");
            write_str(&mut out, &f.message);
            out.push('}');
        }
    }
    out
}
