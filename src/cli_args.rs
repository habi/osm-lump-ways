use clap::Parser;
use clap_verbosity_flag::Verbosity;
use std::path::PathBuf;

use crate::tagfilter;
use crate::TagGrouper;

/// Group OSM ways based on shared tags into GeoJSON MultiLineStrings
///
/// Reads an OSM PBF file, and groups all connected ways together into a MultiLineString
///
/// Use `-f`/-`-tag-filter` to only include ways which match that tag filter
/// `-g`/`--tag-group-k` to group ways by connectiveness *and* whether that tag key is equal.
/// `--min-length-m` Only output way groups with a minimum length of this
/// `--only-longest-n-per-file NUM` Only output the longest `NUM` ways groups.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub(crate) struct Args {
    /// Input PBF filename
    #[arg(short, long, value_name = "FILENAME.osm.pbf")]
    pub input_filename: PathBuf,

    /// Output filename. If `--split-files-by-group` specified, include `%s` for where to place the
    /// group.
    /// Filename .geojson will be GeoJSON, .geojsons will be GeoJSONSeq which is faster for
    /// tippecanoe to read
    #[arg(short, long, value_name = "OUTPUT.geojson[s]")]
    pub output_filename: String,

    /// If the output file already exists, overwrite it. By default, exit if the output already
    /// exists
    #[arg(long)]
    pub overwrite: bool,

    /// Filter input ways by these tags
    ///
    /// Can be specified many times. All values ANDed together. (i.e. way must match all)
    /// Example
    ///   • `-f key` / `-f ∃key`  way has this tag
    ///   • `-f ~key_regex` / `-f ∃~key_regex`  There is a key, which matches this regex
    ///   • `-f ∄key`  way does not has this key
    ///   • `-f key=value`  way has this key and this value
    ///   • `-f key≠value`  way either doesn't have this key, or if it does, it's not equal to value
    ///   • `-f key=value1,value2,…` / -f `key∈value1,value2,…`  way has this key and the value is one of these
    ///   • `-f key≠value1,value2,…` / -f `key∉value1,value2,…`  way either doesn't have this key,
    ///      or if it does, it's not one of these values
    ///   • `-f key~regex` way has this key and the value matches this regex.
    ///     Regexes are case sensitive. Add `(?i)` at start of regex to switch to case insensitive
    ///     (e.g. `-f name~(?i).* street`)
    ///     Regexes match the whole value, `-f name~[Ss]treet` will match `Street`, but not `Main
    ///     Street North` nor `Main Street`. Use `-f name~.*[Ss]treet.*` to match all.
    ///   • `-f F1∨F2∨F3…` logical OR of the other tag filters F1, F2, …
    #[arg(short = 'f', long = "tag-filter", value_name = "FILTER")]
    pub tag_filter: Vec<tagfilter::TagFilter>,

    /// Group by unique values of this key
    ///
    /// Can be specified many times, which will be many groupings.
    /// specify many keys (separated by commas) to use the first set value as the key
    /// `-g name:en,name` → The grouping key will be the the `name:en` key if it's set, else the
    /// `name` key
    #[arg(short = 'g', long = "tag-group-k", value_name = "key1,key2,…")]
    pub tag_group_k: Vec<TagGrouper>,

    /// If grouping by a key, set this to also include ways where there is any unset tag (default
    /// to require all to be set)
    #[arg(long)]
    pub incl_unset_group: bool,

    /// Only include (in the output) lines which are this length (in metres) or more.
    #[arg(long, value_name = "NUMBER")]
    pub min_length_m: Option<f64>,

    /// Only include (in the output) lines which have a dist_to_nearer greater than or equal to
    /// this
    #[arg(long, value_name = "NUMBER", requires = "incl_dist_to_longer")]
    pub min_dist_to_longer_m: Option<f64>,

    /// Per tag group, only include the longest N lines
    #[arg(long, value_name = "N")]
    pub only_longest_n_per_group: Option<usize>,

    /// Per file, only include the longest N lines
    #[arg(long, value_name = "N")]
    pub only_longest_n_per_file: Option<usize>,

    /// When splitting a waygroup into paths, only take the following longest N paths (default:
    /// take all)
    #[arg(long, value_name = "N")]
    pub only_longest_n_splitted_paths: Option<usize>,

    /// Set this to make each group a different filename, or have everything in one file. Default:
    /// false, everything in one file.
    #[arg(long)]
    pub split_files_by_group: bool,

    /// After grouping the ways, split based on longest linear paths
    #[arg(long)]
    pub split_into_single_paths: bool,

    /// Only procoess way groups which include these way ids
    #[arg(long)]
    pub only_these_way_groups: Vec<i64>,

    /// Only procoess way groups which include these node ids
    #[arg(long)]
    pub only_these_way_groups_nodeid: Vec<i64>,

    /// Only procoess way groups where waygroup ID % FIRST = SECOND
    /// Usage: `--only-these-way-groups-divmod 2/0` which only processes way groups where id % 2 ==
    /// 0
    /// Only useful for internal debugging to find problematic way groups
    #[arg(long)]
    pub only_these_way_groups_divmod: Option<String>,

    /// For each output object, calculate the distance (in m) to the nearest, longer object. This
    /// is increadily long for large complicated networks (e.g. waterways), but is reasonable for
    /// named streets.
    #[arg(long, default_value="false", aliases=["incl-distance-to-longer"])]
    pub incl_dist_to_longer: bool,

    /// Include list of OSM wayids for each feature
    /// For each way group, include a JSON property `all_wayids`, a list of all the OSM way ids
    /// that make up this group. Each is a JSON string "w123" (i.e. /^w[0-9]+$/), the same format
    /// `osmium getid` accepts.
    ///
    #[arg(long, action=clap::ArgAction::SetTrue, default_value = "false", aliases=["incl-way-ids", "include-wayids", "include-way-ids"])]
    pub incl_wayids: bool,

    /// Rather than save one MultiLineString per group, save it as many smaller linestrings,
    /// duplication of properties etc
    #[arg(long, default_value = "false")]
    pub save_as_linestrings: bool,

    #[command(flatten)]
    pub verbose: Verbosity<clap_verbosity_flag::InfoLevel>,
}
