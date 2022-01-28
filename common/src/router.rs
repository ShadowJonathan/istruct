use std::collections::HashMap;

use axum;

pub struct VersionedRouter(axum::Router, String, usize, usize);

impl VersionedRouter {
    pub fn new<I: Into<String>>(
        router: axum::Router,
        prefix: I,
        major: usize,
        minor: usize,
    ) -> Self {
        Self(router, prefix.into(), major, minor)
    }
}

pub struct CompositeRouter {
    routers: HashMap<(String, usize, usize), axum::Router>,
}

impl CompositeRouter {
    pub fn new() -> Self {
        Self {
            routers: HashMap::new(),
        }
    }

    pub fn new_with<I>(i: I) -> anyhow::Result<Self>
    where
        I: IntoIterator<Item = VersionedRouter>,
    {
        let mut comp = Self::new();

        for v in i {
            comp.add(v)?;
        }

        Ok(comp)
    }

    pub fn add(
        &mut self,
        VersionedRouter(router, prefix, major, minor): VersionedRouter,
    ) -> anyhow::Result<()> {
        if prefix.contains("/") {
            anyhow::bail!("routing prefixes cannot contain slashes")
        }

        let key = (prefix, major, minor);

        if self.routers.contains_key(&key) {
            anyhow::bail!("Router {:?} already known and added", key);
        }

        self.routers.insert(key, router);

        Ok(())
    }

    // todo: also generate discovery information alongside this
    pub fn assemble(self) -> axum::Router {
        let mut rearranged: HashMap<String, HashMap<usize, Vec<(usize, axum::Router)>>> =
            HashMap::new();

        for ((prefix, major, minor), router) in self.routers {
            rearranged
                .entry(prefix)
                .or_default()
                .entry(major)
                .or_default()
                .push((minor, router));
        }

        let mut amalgamation = axum::Router::new();

        for (prefix, versions) in rearranged {
            for (major, mut routers) in versions {
                let highest: axum::Router;

                if routers.len() == 1 {
                    highest = routers[0].1.clone();
                } else if routers.is_empty() {
                    panic!("can't happen: we immediately push a value after making default vec")
                } else {
                    routers.sort_by_key(|e| e.0);

                    // pick the router with the highest minor value
                    highest = routers
                        .last()
                        .expect("default-then-pushed vec isnt empty")
                        .1
                        .clone();
                }

                for (minor, router) in routers {
                    amalgamation = amalgamation
                        .nest(format!("/{}/v{}.{}", prefix, major, minor).as_str(), router);
                }

                amalgamation =
                    amalgamation.nest(format!("/{}/v{}", prefix, major).as_str(), highest)
            }
        }

        amalgamation
    }
}
