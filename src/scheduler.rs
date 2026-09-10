//! A transparent heuristic scheduler, not FSRS and not a clinically validated model.
use crate::{
    model::{Entry, Skill},
    store::Progress,
};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

const DAY: f64 = 86400.0;

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum Grade {
    Again,
    Hard,
    Good,
    Easy,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Memory {
    pub due: i64,
    pub last: i64,
    /// Days until estimated recall probability reaches 0.9.
    pub stability: f64,
    pub reviews: u32,
    pub lapses: u32,
    #[serde(default)]
    pub cue_chars: u8,
}
impl Default for Memory {
    fn default() -> Self {
        Self {
            due: 0,
            last: 0,
            stability: 1.0,
            reviews: 0,
            lapses: 0,
            cue_chars: 0,
        }
    }
}
impl Memory {
    pub fn recall_estimate(&self, now: i64) -> f64 {
        if self.reviews == 0 {
            return 0.0;
        }
        let days = now.saturating_sub(self.last).max(0) as f64 / DAY;
        0.9_f64
            .powf(days / self.stability.max(0.05))
            .clamp(0.0, 1.0)
    }
    pub fn apply(&mut self, mut grade: Grade, assisted: bool, now: i64) {
        // Hinted success is not counted as unassisted successful retrieval.
        if assisted && matches!(grade, Grade::Good | Grade::Easy) {
            grade = Grade::Hard;
        }
        self.cue_chars = match grade {
            Grade::Again => 2,
            Grade::Hard => self.cue_chars.saturating_sub(1),
            Grade::Good | Grade::Easy => 0,
        };
        let old_recall = self.recall_estimate(now);
        let first = self.reviews == 0;
        self.stability = match grade {
            Grade::Again => (self.stability * 0.4).max(0.3),
            Grade::Hard => {
                if first {
                    0.5
                } else {
                    (self.stability * 1.15).max(0.5)
                }
            }
            Grade::Good => {
                if first {
                    1.0
                } else {
                    self.stability * (2.0 + (1.0 - old_recall).min(0.5))
                }
            }
            Grade::Easy => {
                if first {
                    2.0
                } else {
                    self.stability * 2.8
                }
            }
        }
        .clamp(0.3, 180.0);
        let interval = if grade == Grade::Again {
            120
        } else {
            (self.stability * DAY).round() as i64
        };
        self.last = now;
        self.due = now.saturating_add(interval);
        self.reviews = self.reviews.saturating_add(1);
        if grade == Grade::Again {
            self.lapses = self.lapses.saturating_add(1);
        }
    }
}

#[derive(Clone, Debug)]
pub struct Task {
    pub index: usize,
    pub skill: Skill,
    pub introduce: bool,
}
impl Task {
    pub fn key(&self, deck: &[Entry]) -> String {
        self.skill.key(&deck[self.index].id)
    }
}

pub fn make_queue(
    deck: &[Entry],
    progress: &Progress,
    now: i64,
    today: &str,
    max_new: usize,
) -> VecDeque<Task> {
    let mut due = Vec::new();
    let mut fresh = Vec::new();
    let mut scheduled_bases = std::collections::BTreeSet::new();
    for (index, entry) in deck.iter().enumerate() {
        if progress.settings.topic != "すべて" && entry.tag != progress.settings.topic {
            continue;
        }
        for skill in Skill::ALL {
            if !progress.settings.skills.contains(&skill) {
                continue;
            }
            let key = skill.key(&entry.id);
            if progress.suspended.contains(&entry.id) || progress.deleted_entries.contains(&entry.id) {
                continue;
            }
            match progress.memories.get(&key) {
                Some(m) if m.due <= now => {
                    let overdue =
                        now.saturating_sub(m.due).max(0) as f64 / (m.stability * DAY).max(1.0);
                    due.push((
                        overdue,
                        Task {
                            index,
                            skill,
                            introduce: false,
                        },
                    ));
                }
                None => fresh.push(Task {
                    index,
                    skill,
                    introduce: true,
                }),
                _ => {}
            }
        }
    }
    due.sort_by(|a, b| b.0.total_cmp(&a.0).then(a.1.index.cmp(&b.1.index)));
    // The cap is workload management, not deletion or rescheduling of overdue memories.
    let new_today = progress
        .reviews
        .iter()
        .filter(|r| r.date == today && r.first)
        .count();
    let allowance = if due.len() > 6 {
        0
    } else {
        max_new.saturating_sub(new_today)
    };
    let mut queue: VecDeque<Task> = due.into_iter().take(18).map(|x| x.1).collect();
    // One new skill per base word per session avoids sibling-answer leakage.
    for task in fresh {
        if scheduled_bases.len() >= allowance {
            break;
        }
        let base = &deck[task.index].base;
        if scheduled_bases.contains(base) || queue.iter().any(|t| deck[t.index].base == *base) {
            continue;
        }
        scheduled_bases.insert(base.clone());
        queue.push_back(task);
    }
    // Separate adjacent cards for the same word whenever another word is available.
    let mut result = VecDeque::new();
    let mut last = None;
    while !queue.is_empty() {
        let pos = queue
            .iter()
            .position(|x| Some(deck[x.index].base.as_str()) != last)
            .unwrap_or(0);
        let task = queue.remove(pos).unwrap();
        last = Some(deck[task.index].base.as_str());
        result.push_back(task);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn recoverable_deletion_hides_tasks_and_preserves_learning() {
        let deck = crate::model::parse_deck(crate::model::BUILTIN_DECK).unwrap();
        let mut p = Progress::default();
        let key = Skill::Recall.key(&deck[0].id);
        p.record(key.clone(), Grade::Good, false, "keyboard", 100, "2026-09-05", 5, false);
        let memory = serde_json::to_value(&p.memories).unwrap();
        let reviews = serde_json::to_value(&p.reviews).unwrap();
        p.deleted_entries.insert(deck[0].id.clone());
        let mut p: Progress = serde_json::from_str(&serde_json::to_string(&p).unwrap()).unwrap();
        assert!(make_queue(&deck[..1], &p, 1_000_000, "2026-09-09", 3).is_empty());
        p.deleted_entries.remove(&deck[0].id);
        assert!(!make_queue(&deck[..1], &p, 1_000_000, "2026-09-09", 3).is_empty());
        assert_eq!(serde_json::to_value(&p.memories).unwrap(), memory);
        assert_eq!(serde_json::to_value(&p.reviews).unwrap(), reviews);
    }
    #[test]
    fn spacing_grows_and_lapse_returns_soon() {
        let mut m = Memory::default();
        m.apply(Grade::Good, false, 1_000_000);
        assert_eq!(m.due, 1_086_400);
        m.apply(Grade::Good, false, m.due);
        assert!(m.stability > 2.0);
        m.apply(Grade::Again, false, 2_000_000);
        assert_eq!(m.due, 2_000_120);
        assert_eq!(m.lapses, 1);
    }
    #[test]
    fn assistance_never_gets_easy_interval() {
        let mut m = Memory::default();
        m.apply(Grade::Easy, true, 100);
        assert_eq!(m.stability, 0.5);
    }
    #[test]
    fn clock_going_back_does_not_produce_probability_above_one() {
        let m = Memory {
            last: 100,
            reviews: 1,
            ..Default::default()
        };
        assert_eq!(m.recall_estimate(0), 1.0);
    }
    #[test]
    fn skills_do_not_share_memory() {
        assert_ne!(Skill::Recall.key("sorry"), Skill::Listening.key("sorry"));
    }
    #[test]
    fn failed_word_gets_diminishing_cues() {
        let mut m = Memory::default();
        m.apply(Grade::Again, false, 0);
        assert_eq!(m.cue_chars, 2);
        m.apply(Grade::Good, true, 120);
        assert_eq!(m.cue_chars, 1);
        m.apply(Grade::Good, true, 86400);
        assert_eq!(m.cue_chars, 0);
    }
    #[test]
    fn new_tasks_are_limited_and_siblings_separated() {
        let mut deck = crate::model::parse_deck(crate::model::BUILTIN_DECK).unwrap();
        let mut sibling = deck[0].clone();
        sibling.id = "same-base-another-sense".into();
        deck.insert(1, sibling);
        let mut p = Progress::default();
        let q = make_queue(&deck, &p, 100, "2026-09-05", 3);
        assert_eq!(q.len(), 3);
        assert_eq!(
            q.iter()
                .map(|t| &deck[t.index].base)
                .collect::<std::collections::BTreeSet<_>>()
                .len(),
            3
        );
        p.record(
            Skill::Recall.key(&deck[0].id),
            Grade::Good,
            false,
            "keyboard",
            100,
            "2026-09-05",
            5,
            false,
        );
        assert_eq!(make_queue(&deck, &p, 100, "2026-09-05", 3).len(), 2);
        for entry in deck.iter().skip(2).take(7) {
            p.memories
                .insert(Skill::Recall.key(&entry.id), Memory::default());
        }
        assert!(make_queue(&deck, &p, 100, "2026-09-05", 3)
            .iter()
            .all(|t| !t.introduce));
    }
}
