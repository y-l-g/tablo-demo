//! The blog's data: categories, tags, authors, a post a week, and the comments on them.

use jiff::Zoned;
use tablo::rich_text::from_text;
use toasty::Db;

use super::random::{Rng, date_of, days_ago, person, slug, today_back};
use crate::blog::{Author, Comment, Post, PostCategory, PostTag, Tag};
use crate::shop::Customer;

/// How many people write the posts.
const AUTHORS: u64 = 6;

/// Post categories, authors, their posts, and the comments on them.
pub(super) async fn seed_blog(
    db: &mut Db,
    rng: &mut Rng,
    now: &Zoned,
    customers: &[Customer],
    covers: &[String],
) -> toasty::Result<()> {
    let mut categories = Vec::new();
    for name in POST_CATEGORIES {
        categories.push(
            toasty::create!(PostCategory {
                name: name.to_string(),
                slug: slug(name),
                description: Some(format!("Posts about {name}.")),
                is_visible: rng.boolean(85),
                seo_title: Some(name.to_string()),
                seo_description: Some(format!("{name} articles from the demo blog.")),
                created_at: days_ago(now, rng.below(700) as i64),
            })
            .exec(db)
            .await?,
        );
    }

    let mut tags = Vec::new();
    for name in TAGS {
        tags.push(
            toasty::create!(Tag {
                name: name.to_string(),
            })
            .exec(db)
            .await?,
        );
    }

    let mut authors = Vec::new();
    for index in 0..AUTHORS {
        let name = person(rng);
        let handle = slug(&name);
        let created_at = days_ago(now, rng.between(400, 720) as i64);
        authors.push(
            toasty::create!(Author {
                name: name.clone(),
                email: format!("{handle}.{}@example.com", index + 1),
                bio: Some(format!("{name} writes about building software.")),
                github_handle: Some(handle.clone()),
                twitter_handle: Some(handle),
                created_at,
                updated_at: created_at,
            })
            .exec(db)
            .await?,
        );
    }

    // About one post a week, the newest still drafts and one scheduled for next week.
    for (index, draft) in POSTS.iter().enumerate() {
        let age = ((POSTS.len() - index) * 7) as i64 - 4 + rng.below(3) as i64;
        let created_at = days_ago(now, age);
        let published_at = match POSTS.len() - index {
            1 => Some(today_back(-7)),
            2 => None,
            _ => Some(date_of(created_at)),
        };
        let category = categories
            .iter()
            .find(|category| category.name == draft.category)
            .unwrap_or(&categories[0]);
        let tag_count = rng.between(2, 4);
        let picked = rng.pick_distinct(TAGS, tag_count);
        let post = toasty::create!(Post {
            author_id: Some(rng.pick(&authors).id),
            post_category_id: Some(category.id),
            title: draft.title.to_string(),
            slug: slug(draft.title),
            content: from_text(draft.content),
            published_at,
            image: covers[index % covers.len()].clone(),
            seo_title: Some(draft.title.to_string()),
            seo_description: Some(draft.content.chars().take(120).collect()),
            created_at,
            updated_at: created_at,
        })
        .exec(db)
        .await?;

        for name in picked {
            let tag = tags
                .iter()
                .find(|tag| tag.name == name)
                .expect("every seeded tag is created");
            toasty::create!(PostTag {
                post_id: post.id,
                tag_id: tag.id,
            })
            .exec(db)
            .await?;
        }

        for _ in 0..rng.between(1, 4) {
            toasty::create!(Comment {
                customer_id: Some(rng.pick(customers).id),
                post_id: Some(post.id),
                title: Some(rng.pick(REPLY_TITLES).to_string()),
                content: Some(rng.pick(REPLIES).to_string()),
                is_visible: rng.boolean(85),
                created_at: days_ago(now, rng.below(age.max(1) as u64) as i64),
            })
            .exec(db)
            .await?;
        }
    }
    Ok(())
}

const POST_CATEGORIES: &[&str] = &[
    "Announcements",
    "Design",
    "Engineering",
    "Guides",
    "Performance",
    "Product",
    "Security",
    "Testing",
];

/// The tags the seeded posts carry.
const TAGS: &[&str] = &[
    "Rust",
    "Async",
    "CSS",
    "DevOps",
    "Testing",
    "API",
    "Security",
    "Performance",
    "Architecture",
    "Git",
    "Database",
    "Accessibility",
    "Deployment",
    "Design",
];

/// One blog post the seeder writes, newest last.
struct PostDraft {
    title: &'static str,
    category: &'static str,
    content: &'static str,
}

const POSTS: &[PostDraft] = &[
    PostDraft {
        title: "Shipping a Rust Admin Panel in a Weekend",
        category: "Product",
        content: "A list, a form and a detail page per table, declared next to the models they serve. Most of the weekend went into the data, not the screens.",
    },
    PostDraft {
        title: "Why Our Queues Got Faster",
        category: "Performance",
        content: "Slow work such as sending mail now leaves the request. A worker drains the queue, and the page answers before the mail is written.",
    },
    PostDraft {
        title: "Building Accessible Forms",
        category: "Design",
        content: "Label every input, describe every error, and keep the whole form reachable from the keyboard. Accessibility is a requirement, not a polish step.",
    },
    PostDraft {
        title: "Understanding Database Indexes",
        category: "Engineering",
        content: "An index trades write cost for read speed. Index the columns your filters and joins touch, and read the plan before you add another one.",
    },
    PostDraft {
        title: "A Field Guide to Code Review",
        category: "Guides",
        content: "Review the code, not the author. Keep changes small, say why, and answer within a day so the branch does not drift.",
    },
    PostDraft {
        title: "Designing APIs People Enjoy",
        category: "Engineering",
        content: "Name resources with nouns, answer with consistent errors, and paginate every list. Version before the first breaking change, not after.",
    },
    PostDraft {
        title: "Design Tokens Without the Ceremony",
        category: "Design",
        content: "Colours, radii and shadows live in one place. A theme is a handful of variables, and dark mode is the same handful again.",
    },
    PostDraft {
        title: "Testing Strategies That Scale",
        category: "Testing",
        content: "Many fast unit tests, fewer integration tests, and a handful of end-to-end runs that prove the whole path.",
    },
    PostDraft {
        title: "Deploying With Zero Downtime",
        category: "Engineering",
        content: "Run two environments, move traffic once the new one is healthy, and split every schema change across two releases.",
    },
    PostDraft {
        title: "What We Learned From a Year of Async Rust",
        category: "Engineering",
        content: "Hold no lock across an await, keep futures Send, and let the runtime do the scheduling. The rest is ordinary Rust.",
    },
    PostDraft {
        title: "Two-Factor Sign-In, Explained",
        category: "Security",
        content: "An authenticator app or a mailed code, then recovery codes for the day the phone is lost. Each step is a page of its own.",
    },
    PostDraft {
        title: "Charts That Answer a Question",
        category: "Product",
        content: "A chart earns its place when it answers something: is revenue growing, which week was quiet. Twelve weeks is usually enough to tell.",
    },
    PostDraft {
        title: "The Case for Boring Infrastructure",
        category: "Engineering",
        content: "One database, one binary, one container. Interesting problems should come from the product, not from the deploy.",
    },
    PostDraft {
        title: "Writing Release Notes People Read",
        category: "Announcements",
        content: "Lead with what changed for the reader, link the details, and keep the version number out of the first sentence.",
    },
    PostDraft {
        title: "Securing File Uploads",
        category: "Security",
        content: "Check the extension and the size before storing, serve files from signed links that expire, and never trust a filename.",
    },
    PostDraft {
        title: "Spring Release: Tabs, Tags and Charts",
        category: "Announcements",
        content: "Record pages split into tabs, tags became real lists, and dashboards gained line and bar charts over the last twelve weeks.",
    },
];

const REPLY_TITLES: &[&str] = &["Thanks", "Great write-up", "One question"];

const REPLIES: &[&str] = &[
    "This cleared up something I had been getting wrong for years.",
    "Do you have a follow-up on the migration path?",
    "Bookmarked. The section on indexes alone was worth the read.",
];
