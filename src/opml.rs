use anyhow::Result;
use opml::{Body, Head, Outline, OPML};
use std::io::Write as IoWrite;
use std::path::Path;

pub async fn export(p: &Path) -> Result<()> {
    let global_config = crate::GlobalConfig::load()?;
    let podcasts = crate::Podcast::load_all(&global_config).await?;

    let mut opml = OPML {
        head: Some(Head {
            title: Some("Cringecast Podcast Feeds".to_string()),
            date_created: Some(chrono::Utc::now().to_rfc2822()),
            ..Head::default()
        }),
        ..Default::default()
    };

    let mut outlines = Vec::new();

    for pod in podcasts.iter() {
        outlines.push(Outline {
            text: pod.name().to_owned(),
            r#type: Some("rss".to_string()),
            xml_url: Some(pod.config().url.clone()),
            title: Some(pod.name().to_owned()),
            ..Outline::default()
        });
    }

    opml.body = Body { outlines };

    let xml_string = opml.to_string()?;

    std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(p)?
        .write_all(xml_string.as_bytes())?;

    Ok(())
}

pub fn import(p: &Path) -> Result<()> {
    let opml_string = std::fs::read_to_string(p)?;
    let opml = opml::OPML::from_str(&opml_string)?;

    let mut podcasts = vec![];

    for podcast in opml.body.outlines.into_iter() {
        let title = {
            let title = podcast.title.unwrap_or(podcast.text);

            if title.is_empty() {
                None
            } else {
                Some(title)
            }
        };

        let (title, url) = match (title, podcast.xml_url) {
            (None, None) => {
                eprintln!("importing failed due to feed with missing title and url");
                std::process::exit(1);
            }
            (Some(title), None) => {
                eprintln!(
                    "importing failed due to following podcast missing its' url: {}",
                    title
                );
                std::process::exit(1);
            }
            (None, Some(url)) => {
                eprintln!(
                    "importing failed due to podcast with following url missing a title: {}",
                    url
                );
                std::process::exit(1);
            }
            (Some(title), Some(url)) => (title, url),
        };

        podcasts.push((title, url));
    }

    if podcasts.is_empty() {
        eprintln!("no podcasts found.");
    } else {
        crate::utils::append_podcasts(podcasts)?;
    }

    Ok(())
}
