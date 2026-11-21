// Derived from Apache Pekko 2.0.0 actor/src/main/scala/org/apache/pekko/util/WildcardIndex.scala (Apache-2.0).
//! Twin of `WildcardWinnersOracle.java` (Q-UTIL-21): which of several overlapping `prefix*` entries a `WildcardIndex` returns.

use rakka_actor::util::wildcard_index::WildcardIndex;

/// One input line `prefix,prefix,...|probe`: the prefixes are inserted in that order as `[prefix*, "x"]` with the prefix as
/// data, the probe is looked up as `[probe, "x"]`; the output is the prefix found or `none`.
pub fn winner(line: &str) -> String {
    let (prefixes, probe) = line.rsplit_once('|').expect("a line is prefixes|probe");
    let mut index = WildcardIndex::new()
        .insert(&["q"], "anchor".to_owned())
        .expect("a valid path");
    for prefix in prefixes.split(',') {
        index = index
            .insert(&[&format!("{prefix}*"), "x"], prefix.to_owned())
            .expect("a valid path");
    }
    index
        .find([probe, "x"])
        .cloned()
        .unwrap_or_else(|| "none".to_owned())
}
