use clap::StructOpt;
use didkit::{Error, HTTPDIDResolver, SeriesResolver, DID_METHODS};
use ssi::jsonld::ContextLoader;
use std::collections::HashMap;

#[derive(StructOpt, Debug, Clone, Default)]
pub struct ResolverOptions {
    #[clap(env, short = 'r', long, parse(from_str = HTTPDIDResolver::new))]
    /// Fallback DID Resolver HTTP(S) endpoint, for non-built-in DID methods.
    pub did_resolver: Option<HTTPDIDResolver>,
    #[clap(env, short = 'R', long, parse(from_str = HTTPDIDResolver::new))]
    /// Override DID Resolver HTTP(S) endpoint, for all DID methods.
    pub did_resolver_override: Option<HTTPDIDResolver>,
}

impl ResolverOptions {
    pub fn to_resolver<'a>(&'a self) -> SeriesResolver<'a> {
        let mut resolvers = vec![DID_METHODS.to_resolver()];
        if let Some(http_did_resolver) = &self.did_resolver {
            resolvers.push(http_did_resolver);
        }
        if let Some(http_did_resolver) = &self.did_resolver_override {
            resolvers.insert(0, http_did_resolver);
        }
        SeriesResolver { resolvers }
    }
}

#[derive(StructOpt, Clone, Debug, Default)]
pub struct ContextLoaderOptions {
    #[clap(env, long)]
    /// Indicate that the default, built-in JSONLD context objects should not be used during JSONLD
    /// context resolution for signing and verification.  Default behavior is to use the built-in
    /// context objects.
    pub disable_default_contexts: bool,
    #[clap(env, long)]
    /// Specifies additional JSONLD context objects to be used during JSONLD context resolution
    /// for signing and verification.  If specified, it should have the form
    /// `[{"url": "...", "docBodyFilePath": "..."}, {"url": "...", "docBodyFilePath": "..."}, ...]`
    pub additional_contexts: Option<AdditionalContexts>,
}

impl ContextLoaderOptions {
    pub fn to_context_loader(&self) -> ContextLoader {
        let context_loader = if self.disable_default_contexts {
            ContextLoader::empty()
        } else {
            ContextLoader::default()
        };

        let context_loader = match self.additional_contexts.as_ref() {
            Some(additional_contexts) => {
                let mut preparsed_context_map = HashMap::new();
                for context_loader_entry in additional_contexts.0.iter() {
                    // Check for collision
                    if preparsed_context_map.contains_key(&context_loader_entry.url) {
                        let r: Result<(), Error> =
                            Err(Error::InvalidContextLoaderEntry(
                                format!("Collision in additional-contexts URL: {}", context_loader_entry.url)
                            ));
                        r.unwrap();
                    }
                    // Read the doc body from file.
                    let doc_body =
                        std::fs::read_to_string(&context_loader_entry.doc_body_file_path)
                            .or_else(|e| Err(Error::InvalidContextLoaderEntry(
                                format!(
                                    "could not read doc body from path: {:?}; error was {}",
                                    context_loader_entry.doc_body_file_path,
                                    e
                                )
                            ))).unwrap();
                    preparsed_context_map.insert(context_loader_entry.url.clone(), doc_body);
                }
                context_loader
                    .with_context_map_from(preparsed_context_map)
                    .or_else(|e| Err(Error::InvalidContextLoaderEntry(
                        format!("error while initializing ContextLoader; error was {}", e)
                    ))).unwrap()
            }
            None => context_loader,
        };

        context_loader
    }
}

#[derive(Clone, Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContextLoaderEntry {
    pub url: String,
    pub doc_body_file_path: String,
}

impl std::str::FromStr for ContextLoaderEntry {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(serde_json::from_str(s)?)
    }
}

#[derive(Clone, Debug, serde::Deserialize)]
pub struct AdditionalContexts(pub(crate) Vec<ContextLoaderEntry>);

impl std::str::FromStr for AdditionalContexts {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(serde_json::from_str(s)?)
    }
}
