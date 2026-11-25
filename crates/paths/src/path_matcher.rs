use crate::{
    PathStyle,
    rel_path::{RelPath, RelPathBuf},
};
use globset::{GlobBuilder, GlobSet, GlobSetBuilder};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PathMatcherSources {
    raw: String,
    path: RelPathBuf,
    trailing_separator: bool,
}

impl PathMatcherSources {
    pub fn new(raw: String, path: RelPathBuf, trailing_separator: bool) -> Self {
        Self {
            raw,
            path,
            trailing_separator,
        }
    }

    pub fn raw(&self) -> &str {
        self.raw.as_str()
    }

    pub fn path(&self) -> &RelPath {
        self.path.as_rel_path()
    }

    pub fn trailing_separator(&self) -> bool {
        self.trailing_separator
    }
}

#[derive(Clone, Debug)]
pub struct PathMatcher {
    sources: Vec<PathMatcherSources>,
    glob: GlobSet,
    path_style: PathStyle,
}

impl PartialEq for PathMatcher {
    fn eq(&self, other: &Self) -> bool {
        self.sources.eq(&other.sources)
    }
}

impl Eq for PathMatcher {}

impl PathMatcher {
    pub fn new(
        globs: impl IntoIterator<Item = impl AsRef<str>>,
        path_style: PathStyle,
    ) -> Result<Self, globset::Error> {
        let globs = globs
            .into_iter()
            .map(|as_str| {
                GlobBuilder::new(as_str.as_ref())
                    .backslash_escape(path_style.is_posix())
                    .build()
            })
            .collect::<Result<Vec<_>, _>>()?;

        let sources = globs
            .iter()
            .filter_map(|glob| {
                let glob = glob.glob();

                Some(PathMatcherSources::new(
                    glob.to_string(),
                    RelPath::new(glob.as_ref(), path_style)
                        .ok()
                        .map(std::borrow::Cow::into_owned)?,
                    glob.ends_with(path_style.separators_ch()),
                ))
            })
            .collect();

        let mut glob_builder = GlobSetBuilder::new();
        for single_glob in globs {
            glob_builder.add(single_glob);
        }
        let glob = glob_builder.build()?;

        Ok(PathMatcher {
            glob,
            sources,
            path_style,
        })
    }

    pub fn sources(&self) -> impl Iterator<Item = &PathMatcherSources> {
        self.sources.iter()
    }

    pub fn is_match<P: AsRef<RelPath>>(&self, other: P) -> bool {
        if self
            .sources
            .iter()
            .any(|source| other.as_ref().starts_with(source.path()) || other.as_ref().ends_with(source.path()))
        {
            return true;
        }
        let other_path = other.as_ref().display(self.path_style);

        if self.glob.is_match(&*other_path) {
            return true;
        }

        self.glob
            .is_match(other_path.into_owned() + self.path_style.primary_separator())
    }
}

impl Default for PathMatcher {
    fn default() -> Self {
        Self {
            path_style: PathStyle::local(),
            glob: GlobSet::empty(),
            sources: vec![],
        }
    }
}
