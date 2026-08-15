use crate::{
    dataset::Dataset,
    types::{ArticleMeta, ArticleRef, Challenge, DifficultyPreset, GameMode, Settings},
};
use rand::{prelude::IndexedRandom, rng, Rng};
use std::collections::HashSet;
use unicode_normalization::UnicodeNormalization;

#[derive(Debug, Clone)]
pub struct DifficultyConfig {
    pub lexical_weight: f32,
    pub topic_weight: f32,
    pub graph_weight: f32,
    pub community_weight: f32,
    pub popularity_weight: f32,
    pub navigability_weight: f32,
    pub min_difficulty: f32,
    pub max_difficulty: f32,
}
impl Default for DifficultyConfig {
    fn default() -> Self {
        Self {
            lexical_weight: 0.15,
            topic_weight: 0.20,
            graph_weight: 0.30,
            community_weight: 0.20,
            popularity_weight: 0.05,
            navigability_weight: 0.10,
            min_difficulty: 0.68,
            max_difficulty: 0.86,
        }
    }
}

impl DifficultyConfig {
    fn for_preset(preset: DifficultyPreset) -> Self {
        let mut config = Self::default();
        if preset == DifficultyPreset::Evil {
            config.min_difficulty = 0.82;
            config.max_difficulty = 0.94;
        }
        config
    }
}

pub fn normalize_title(title: &str) -> String {
    title
        .nfkc()
        .collect::<String>()
        .replace('_', " ")
        .to_lowercase()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}
fn tokens(title: &str) -> HashSet<String> {
    normalize_title(title)
        .split(|c: char| !c.is_alphanumeric())
        .filter(|s| s.len() > 1)
        .map(str::to_string)
        .collect()
}
fn lexical_distance(a: &str, b: &str) -> f32 {
    let a = tokens(a);
    let b = tokens(b);
    let union = a.union(&b).count();
    if union == 0 {
        1.0
    } else {
        1.0 - a.intersection(&b).count() as f32 / union as f32
    }
}
fn graph_distance(a: &ArticleMeta, b: &ArticleMeta) -> f32 {
    let same = a
        .graph_signature
        .iter()
        .zip(b.graph_signature)
        .filter(|(x, y)| **x == *y)
        .count();
    1.0 - same as f32 / 4.0
}
fn topic_distance(a: &ArticleMeta, b: &ArticleMeta) -> f32 {
    if a.topic_mask & b.topic_mask == 0 {
        1.0
    } else {
        0.25
    }
}
fn popularity_balance(a: &ArticleMeta, b: &ArticleMeta) -> f32 {
    let hi = a.in_degree.max(b.in_degree).max(1) as f32;
    let lo = a.in_degree.min(b.in_degree).max(1) as f32;
    (lo / hi).sqrt()
}
fn navigability(a: &ArticleMeta, b: &ArticleMeta) -> f32 {
    ((a.out_degree.min(100) + b.out_degree.min(100)) as f32 / 200.0).clamp(0.0, 1.0)
}

pub fn pair_is_viable(a: &ArticleMeta, b: &ArticleMeta) -> bool {
    if a.id == b.id
        || a.is_redirect
        || b.is_redirect
        || a.is_disambiguation
        || b.is_disambiguation
        || a.out_degree < 8
        || b.out_degree < 8
    {
        return false;
    }
    let lexical = lexical_distance(&a.normalized_title, &b.normalized_title);
    if lexical < 0.55 {
        return false;
    }
    // Single-topic pairs are usually flat rather than intriguingly distant
    // (city → ocean, composer → composer, country → city). Multi-topic pages
    // can still form interesting related challenges.
    if a.topic_mask == b.topic_mask && a.topic_mask.count_ones() == 1 {
        return false;
    }
    // A near-identical graph neighborhood/community denotes an obvious direct conceptual neighbor.
    !(a.community_id == b.community_id && graph_distance(a, b) < 0.30 && topic_distance(a, b) < 0.5)
}
pub fn difficulty(a: &ArticleMeta, b: &ArticleMeta, c: &DifficultyConfig) -> f32 {
    let community = if a.community_id == b.community_id {
        0.15
    } else {
        1.0
    };
    (lexical_distance(&a.normalized_title, &b.normalized_title) * c.lexical_weight
        + topic_distance(a, b) * c.topic_weight
        + graph_distance(a, b) * c.graph_weight
        + community * c.community_weight
        + popularity_balance(a, b) * c.popularity_weight
        + navigability(a, b) * c.navigability_weight)
        .clamp(0.0, 1.0)
}

pub fn click_limit(score: f32, c: &DifficultyConfig) -> u32 {
    let span = (c.max_difficulty - c.min_difficulty).max(f32::EPSILON);
    let normalized = ((score - c.min_difficulty) / span).clamp(0.0, 1.0);
    (3.0 + normalized * 7.0).round() as u32
}

pub fn generate(
    dataset: &Dataset,
    previous: Option<&Challenge>,
    settings: &Settings,
) -> Result<Challenge, String> {
    // Keep runtime memory bounded even when the production database contains
    // millions of titles. The indexed database sampler wraps at the ID range.
    let articles = dataset.eligible_articles(512)?;
    if articles.len() < 2 {
        return Err("Article database has too few eligible pages".into());
    }
    let mut rng = rng();
    let preset = if settings.default_mode == GameMode::Evil {
        DifficultyPreset::Evil
    } else {
        settings.default_difficulty
    };
    let config = DifficultyConfig::for_preset(preset);
    let prior_ids = previous.map(|p| [p.start.id, p.target.id]);
    let starts: Vec<_> = articles
        .iter()
        .filter(|a| prior_ids.is_none_or(|ids| !ids.contains(&a.id)))
        .collect();
    let start = starts.choose(&mut rng).copied().unwrap_or(&articles[0]);
    let ideal: f32 = if preset == DifficultyPreset::Evil {
        rng.random_range(0.86..=0.94)
    } else {
        rng.random_range(0.72..=0.84)
    };
    let mut best: Option<(&ArticleMeta, f32)> = None;
    for target in articles.choose_multiple(&mut rng, articles.len().min(192)) {
        if prior_ids.is_some_and(|ids| ids.contains(&target.id)) || !pair_is_viable(start, target) {
            continue;
        }
        let score = difficulty(start, target, &config);
        let band_penalty = if score < config.min_difficulty {
            config.min_difficulty - score
        } else if score > config.max_difficulty {
            score - config.max_difficulty
        } else {
            0.0
        };
        let quality = (score - ideal).abs() + band_penalty * 2.0;
        if best.is_none_or(|(_, q)| quality < q) {
            best = Some((target, quality));
        }
    }
    let target = best
        .map(|x| x.0)
        .or_else(|| articles.iter().find(|a| a.id != start.id))
        .ok_or("No viable target found")?;
    let score = difficulty(start, target, &config);
    let generated_limit = click_limit(score, &config);
    let configured_limit = if settings.default_mode == GameMode::MaxClicks {
        settings.max_clicks
    } else {
        generated_limit
    };
    let time_limit_seconds =
        if settings.default_mode == GameMode::TimeLimit && settings.time_limit_seconds > 0 {
            Some(settings.time_limit_seconds)
        } else {
            None
        };
    let first_target = ArticleRef {
        id: target.id,
        title: target.title.clone(),
    };
    let mut targets = vec![first_target.clone()];
    if settings.default_mode == GameMode::Gauntlet {
        let mut used = HashSet::from([start.id, target.id]);
        let mut cursor = target;
        while targets.len() < 5 {
            let next = articles
                .choose_multiple(&mut rng, articles.len().min(256))
                .filter(|candidate| {
                    !used.contains(&candidate.id) && pair_is_viable(cursor, candidate)
                })
                .max_by(|a, b| {
                    difficulty(cursor, a, &config)
                        .partial_cmp(&difficulty(cursor, b, &config))
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
                .or_else(|| {
                    articles
                        .iter()
                        .find(|candidate| !used.contains(&candidate.id))
                })
                .ok_or("No viable Gauntlet target found")?;
            used.insert(next.id);
            targets.push(ArticleRef {
                id: next.id,
                title: next.title.clone(),
            });
            cursor = next;
        }
    }
    Ok(Challenge {
        start: ArticleRef {
            id: start.id,
            title: start.title.clone(),
        },
        target: first_target,
        click_limit: configured_limit,
        time_limit_seconds,
        difficulty: score,
        mode: settings.default_mode,
        targets,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    fn meta(id: u32, title: &str, topic: u32, community: u32, sig: [u32; 4]) -> ArticleMeta {
        ArticleMeta {
            id,
            title: title.into(),
            normalized_title: normalize_title(title),
            is_redirect: false,
            is_disambiguation: false,
            in_degree: 50,
            out_degree: 30,
            topic_mask: topic,
            community_id: community,
            graph_signature: sig,
        }
    }
    #[test]
    fn identical_titles_rejected() {
        let a = meta(1, "Roman Empire", 1, 1, [1, 2, 3, 4]);
        let mut b = a.clone();
        b.id = 2;
        assert!(!pair_is_viable(&a, &b));
    }
    #[test]
    fn distant_metadata_scores_higher() {
        let a = meta(1, "Quantum mechanics", 1, 1, [1, 2, 3, 4]);
        let near = meta(2, "Classical mechanics", 1, 1, [1, 2, 3, 8]);
        let far = meta(3, "Jazz", 2, 9, [9, 8, 7, 6]);
        assert!(
            difficulty(&a, &far, &Default::default()) > difficulty(&a, &near, &Default::default())
        );
    }
    #[test]
    fn same_single_topic_pairs_are_rejected() {
        let a = meta(1, "Atlantic Ocean", 1, 1, [1, 2, 3, 4]);
        let b = meta(2, "Madrid", 1, 2, [5, 6, 7, 8]);
        assert!(!pair_is_viable(&a, &b));
    }
    #[test]
    fn normalization_handles_unicode_and_underscores() {
        assert_eq!(normalize_title("  Café_du Monde "), "café du monde");
    }
    #[test]
    fn click_limits_scale_from_three_to_ten() {
        let c = DifficultyConfig::default();
        assert_eq!(click_limit(c.min_difficulty, &c), 3);
        assert_eq!(click_limit(c.max_difficulty, &c), 10);
        assert!(click_limit(0.80, &c) > click_limit(0.72, &c));
    }
    #[test]
    fn evil_band_is_harder_than_normal() {
        let normal = DifficultyConfig::for_preset(DifficultyPreset::Normal);
        let evil = DifficultyConfig::for_preset(DifficultyPreset::Evil);
        assert!(evil.min_difficulty > normal.min_difficulty);
        assert!(evil.max_difficulty > normal.max_difficulty);
    }

    #[test]
    fn thousand_challenge_batch_terminates_inside_click_range() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../data/articles.sqlite");
        let dataset = Dataset::open(&path).expect("development dataset");
        let settings = Settings::default();
        let mut total = 0.0_f64;
        for _ in 0..1_000 {
            let challenge = generate(&dataset, None, &settings).expect("valid challenge");
            assert_ne!(challenge.start.id, challenge.target.id);
            assert!((3..=10).contains(&challenge.click_limit));
            total += challenge.difficulty as f64;
        }
        let average = total / 1_000.0;
        assert!((0.0..=1.0).contains(&average));
    }
}
